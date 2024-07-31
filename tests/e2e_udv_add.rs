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
    let mut file = File::create(&test_file_path).unwrap();
    writeln!(file, "Test content").unwrap();

    // Run udv add
    let output = run_udv_add(&binary_path, &temp_dir_path, "test_file.txt");
    assert!(output.status.success(), "udv add failed");

    // Check .dvc file creation
    let dvc_file_path = temp_dir_path.join("test_file.txt.dvc");
    assert!(dvc_file_path.exists(), ".dvc file was not created");

    // Check file moved to cache
    let cache_dir = temp_dir_path.join(".udv").join("cache");
    assert!(cache_dir.exists(), "Cache directory was not created");

    // We can't check the exact file name in cache without knowing the hash algorithm,
    // but we can check that the cache is not empty
    assert!(
        fs::read_dir(cache_dir).unwrap().next().is_some(),
        "Cache directory is empty"
    );

    // Check .gitignore update
    let gitignore_path = temp_dir_path.join(".gitignore");
    let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
    assert!(
        gitignore_content.contains("test_file.txt"),
        "File not added to .gitignore"
    );
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
    File::create(test_dir_path.join("file1.txt")).unwrap();
    File::create(test_dir_path.join("file2.txt")).unwrap();

    // Run udv add
    let output = run_udv_add(&binary_path, &temp_dir_path, "test_dir");
    assert!(output.status.success(), "udv add failed for directory");

    // Check .dvc file creation
    let dvc_file_path = temp_dir_path.join("test_dir.dvc");
    assert!(
        dvc_file_path.exists(),
        ".dvc file was not created for directory"
    );

    // Check files moved to cache
    let cache_dir = temp_dir_path.join(".udv").join("cache");
    assert!(cache_dir.exists(), "Cache directory was not created");
    assert!(
        fs::read_dir(cache_dir).unwrap().count() >= 2,
        "Not all files were cached"
    );

    // Check .gitignore update
    let gitignore_path = temp_dir_path.join(".gitignore");
    let gitignore_content = fs::read_to_string(gitignore_path).unwrap();
    assert!(
        gitignore_content.contains("test_dir"),
        "Directory not added to .gitignore"
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
