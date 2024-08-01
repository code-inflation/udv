use pretty_assertions::assert_eq;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

// Reuse setup functions from the previous tests
fn setup_temp_dir() -> PathBuf {
    tempdir().unwrap().into_path()
}

fn init_git_repo(temp_dir_path: &PathBuf) {
    Command::new("git")
        .args(["init", temp_dir_path.to_str().unwrap()])
        .output()
        .expect("Failed to initialize git repository for testing");
}

fn get_binary_path() -> PathBuf {
    let binary_path = std::env::current_dir().unwrap().join("target/debug/udv");
    assert!(
        binary_path.exists(),
        "Binary does not exist: {:?}",
        binary_path
    );
    binary_path
}

fn run_udv_init(binary_path: &PathBuf, temp_dir_path: &PathBuf) {
    let output = Command::new(binary_path)
        .arg("init")
        .current_dir(temp_dir_path)
        .output()
        .expect("Failed to execute udv init command");
    assert!(output.status.success(), "udv init failed");
}

fn run_udv_add(
    binary_path: &PathBuf,
    temp_dir_path: &PathBuf,
    file_path: &str,
) -> std::process::Output {
    Command::new(binary_path)
        .args(["add", file_path])
        .current_dir(temp_dir_path)
        .output()
        .expect("Failed to execute udv add command")
}

#[test]
fn test_udv_add_single_file() {
    let binary_path = get_binary_path();
    let temp_dir_path = setup_temp_dir();
    init_git_repo(&temp_dir_path);
    run_udv_init(&binary_path, &temp_dir_path);

    // Create a test file
    let test_file_path = temp_dir_path.join("test_file.txt");
    let mut file = File::create(test_file_path).unwrap();
    writeln!(file, "Test content").unwrap();

    // Run udv add
    let output = run_udv_add(&binary_path, &temp_dir_path, "test_file.txt");
    assert!(output.status.success(), "udv add failed");

    // Check .dvc file creation and content
    let dvc_file_path = temp_dir_path.join("test_file.txt.dvc");
    assert!(dvc_file_path.exists(), ".dvc file was not created");

    let dvc_content = fs::read_to_string(dvc_file_path).expect("Failed to read .dvc file");
    let dvc_json: serde_json::Value =
        serde_json::from_str(&dvc_content).expect("Failed to parse .dvc JSON");

    assert_eq!(
        dvc_json["algo"], "sha256",
        "Incorrect hash algorithm in .dvc file"
    );
    assert!(
        dvc_json["hash"].is_string(),
        "Hash is not a string in .dvc file"
    );
    assert!(
        dvc_json["size_bytes"].is_u64(),
        "Size is not a u64 in .dvc file"
    );

    // Check file moved to cache
    let cache_dir = temp_dir_path.join(".udv").join("cache");
    assert!(cache_dir.exists(), "Cache directory was not created");

    // Check .gitignore update
    let gitignore_path = temp_dir_path.join(".gitignore");
    let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
    assert!(
        gitignore_content.contains("test_file.txt"),
        "File not added to .gitignore"
    );
}

#[test]
fn test_udv_add_nonexistent_file() {
    let binary_path = get_binary_path();
    let temp_dir_path = setup_temp_dir();
    init_git_repo(&temp_dir_path);
    run_udv_init(&binary_path, &temp_dir_path);

    // Run udv add on a non-existent file
    let output = run_udv_add(&binary_path, &temp_dir_path, "nonexistent_file.txt");
    assert!(
        !output.status.success(),
        "udv add unexpectedly succeeded for non-existent file"
    );
}

#[test]
fn test_udv_add_already_tracked_file() {
    let binary_path = get_binary_path();
    let temp_dir_path = setup_temp_dir();
    init_git_repo(&temp_dir_path);
    run_udv_init(&binary_path, &temp_dir_path);

    // Create and add a test file
    let test_file_path = temp_dir_path.join("test_file.txt");
    File::create(&test_file_path).unwrap();
    run_udv_add(&binary_path, &temp_dir_path, "test_file.txt");

    // Try to add the same file again
    let output = run_udv_add(&binary_path, &temp_dir_path, "test_file.txt");
    assert!(
        output.status.success(),
        "udv add failed for already tracked file"
    );
    // You might want to check the output to see if it contains a warning message
}

#[test]
fn test_udv_add_directory() {
    let binary_path = get_binary_path();
    let temp_dir_path = setup_temp_dir();
    init_git_repo(&temp_dir_path);
    run_udv_init(&binary_path, &temp_dir_path);

    // Create a test directory with files
    let test_dir_path = temp_dir_path.join("test_dir");
    fs::create_dir(&test_dir_path).unwrap();

    // Create files with unique content
    let file1_path = test_dir_path.join("file1.txt");
    let content1 = "Content of file 1";
    fs::write(&file1_path, content1).unwrap();

    let file2_path = test_dir_path.join("file2.txt");
    let content2 = "Different content for file 2";
    fs::write(&file2_path, content2).unwrap();

    // Create a subdirectory with a file
    let sub_dir_path = test_dir_path.join("sub_dir");
    fs::create_dir(&sub_dir_path).unwrap();
    let file3_path = sub_dir_path.join("file3.txt");
    let content3 = "Unique content for file 3 in subdirectory";
    fs::write(&file3_path, content3).unwrap();

    // Run udv add
    let output = run_udv_add(&binary_path, &temp_dir_path, "test_dir");
    assert!(output.status.success(), "udv add failed for directory");

    // Check .dvc file creation for each file
    assert!(
        file1_path.with_extension("txt.dvc").exists(),
        ".dvc file was not created for file1.txt"
    );
    assert!(
        file2_path.with_extension("txt.dvc").exists(),
        ".dvc file was not created for file2.txt"
    );
    assert!(
        file3_path.with_extension("txt.dvc").exists(),
        ".dvc file was not created for file3.txt"
    );

    // Check .dvc file creation and content for each file
    for file_path in &[&file1_path, &file2_path, &file3_path] {
        let dvc_file_path = file_path.with_extension("txt.dvc");
        assert!(
            dvc_file_path.exists(),
            ".dvc file was not created for {:?}",
            file_path
        );

        let dvc_content = fs::read_to_string(&dvc_file_path).expect("Failed to read .dvc file");
        let dvc_json: serde_json::Value =
            serde_json::from_str(&dvc_content).expect("Failed to parse .dvc JSON");

        assert_eq!(
            dvc_json["algo"], "sha256",
            "Incorrect hash algorithm in .dvc file"
        );
        assert!(
            dvc_json["hash"].is_string(),
            "Hash is not a string in .dvc file"
        );
        assert!(
            dvc_json["size_bytes"].is_u64(),
            "Size is not a u64 in .dvc file"
        );
    }

    // Check .gitignore update
    let gitignore_path = temp_dir_path.join(".gitignore");
    let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
    assert!(
        gitignore_content.contains("test_dir/file1.txt")
            && gitignore_content.contains("test_dir/file2.txt")
            && gitignore_content.contains("test_dir/sub_dir/file3.txt"),
        "Not all files were added to .gitignore"
    );
}

#[test]
fn test_udv_add_directory_excludes_udv_folder() {
    let binary_path = get_binary_path();
    let temp_dir_path = setup_temp_dir();
    init_git_repo(&temp_dir_path);
    run_udv_init(&binary_path, &temp_dir_path);

    // Create a test directory with files
    let test_dir_path = temp_dir_path.join("test_dir");
    fs::create_dir(&test_dir_path).unwrap();

    // Create a file in the test directory
    let file1_path = test_dir_path.join("file1.txt");
    let content1 = "Content of file 1";
    fs::write(&file1_path, content1).unwrap();

    // Create a .udv folder inside the test directory
    let udv_dir_path = test_dir_path.join(".udv");
    fs::create_dir(&udv_dir_path).unwrap();

    // Create a file inside the .udv folder
    let udv_file_path = udv_dir_path.join("udv_file.txt");
    let udv_content = "This file should not be tracked";
    fs::write(&udv_file_path, udv_content).unwrap();

    // Run udv add on the test directory
    let output = run_udv_add(&binary_path, &temp_dir_path, "test_dir");
    assert!(output.status.success(), "udv add failed for directory");

    // Check that no .dvc file was created for the file in the .udv folder
    assert!(
        !udv_file_path.with_extension("txt.dvc").exists(),
        ".dvc file was incorrectly created for a file in the .udv folder"
    );

    // Check files moved to cache
    let cache_dir = temp_dir_path.join(".udv").join("cache");
    assert!(cache_dir.exists(), "Cache directory was not created");

    // Calculate SHA256 hash for the file in .udv folder
    let udv_hash = calculate_sha256(udv_content);

    // Check that the file from .udv folder is not in the cache
    let udv_cache_file = cache_dir.join(&udv_hash[..2]).join(&udv_hash[2..]);
    assert!(
        !udv_cache_file.exists(),
        "File from .udv folder was incorrectly cached"
    );

    // Check .gitignore update
    let gitignore_path = temp_dir_path.join(".gitignore");
    let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
    assert!(
        gitignore_content.contains("test_dir/file1.txt"),
        "file1.txt was not added to .gitignore"
    );
    assert!(
        !gitignore_content.contains("test_dir/.udv/udv_file.txt"),
        "File from .udv folder was incorrectly added to .gitignore"
    );
}

fn calculate_sha256(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}
