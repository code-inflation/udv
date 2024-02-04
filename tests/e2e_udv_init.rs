use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

fn setup_temp_dir() -> PathBuf {
    tempdir().unwrap().into_path()
}

fn init_git_repo(temp_dir_path: PathBuf) {
    Command::new("git")
        .args(["init", temp_dir_path.to_str().unwrap()])
        .output()
        .expect("Failed to initialize git repository for testing");
}

fn get_binary_path() -> std::path::PathBuf {
    let binary_path = std::env::current_dir().unwrap().join("target/debug/udv");
    assert!(
        binary_path.exists(),
        "Binary does not exist: {:?}",
        binary_path
    );
    binary_path
}

fn run_udv_init(binary_path: PathBuf, temp_dir_path: PathBuf) -> std::process::Output {
    let output = Command::new(binary_path)
        .arg("init")
        .current_dir(temp_dir_path)
        .output()
        .expect("Failed to execute command");
    output
}

#[test]
fn test_udv_init() {
    let binary_path = get_binary_path();
    let temp_dir_path = setup_temp_dir();
    init_git_repo(temp_dir_path.clone());

    // Run "udv init"
    let output = run_udv_init(binary_path, temp_dir_path.clone());

    assert!(output.status.success(), "Command did not run successfully");

    // Check existence of .udv directory
    let udv_dir = temp_dir_path.join(".udv");
    assert!(udv_dir.exists(), "udv directory was not created");

    // Check existence and content of .udv/.gitignore
    let gitignore_path = udv_dir.join(".gitignore");
    assert!(
        gitignore_path.exists(),
        ".gitignore file was not created in .udv directory"
    );
    let gitignore_contents =
        fs::read_to_string(gitignore_path).expect("Failed to read .gitignore file");
    let expected_gitignore_contents = "/config.local\n/tmp\n/cache";
    assert_eq!(
        gitignore_contents, expected_gitignore_contents,
        ".gitignore contents do not match expected"
    );

    // Check existence and contesnts of .udv/config
    let config_path = udv_dir.join("config");
    assert!(
        config_path.exists(),
        "config file was not created in .udv directory"
    );

    let config_contents = fs::read_to_string(config_path).expect("Failed to read config file");
    let expected_config_contents = "";
    assert_eq!(
        config_contents, expected_config_contents,
        "config file contents do not match expected"
    );
}

#[test]
fn test_udv_init_existing_dvc_folder() {
    let binary_path = get_binary_path();
    let temp_dir_path = setup_temp_dir();
    init_git_repo(temp_dir_path.clone());

    // Create a .dvc directory to simulate an existing DVC configuration
    let dvc_dir_path = temp_dir_path.join(".dvc");
    fs::create_dir(dvc_dir_path).expect("Failed to create .dvc directory for testing");

    // Run "udv init"
    let output = run_udv_init(binary_path, temp_dir_path);

    assert!(
        !output.status.success(),
        "udv init unexpectedly succeeded despite existing .dvc folder"
    );
}

#[test]
fn test_udv_init_existing_udv_folder() {
    let binary_path = get_binary_path();
    let temp_dir_path = setup_temp_dir();
    init_git_repo(temp_dir_path.clone());

    // Create a .dvc directory to simulate an existing uDV configuration
    let dvc_dir_path = temp_dir_path.join(".udv");
    fs::create_dir(dvc_dir_path).expect("Failed to create .udv directory for testing");

    // Run "udv init"
    let output = run_udv_init(binary_path, temp_dir_path);

    assert!(
        !output.status.success(),
        "udv init unexpectedly succeeded despite existing .udv folder"
    );
}

#[test]
fn test_udv_init_not_in_git_repo() {
    let binary_path = get_binary_path();
    let temp_dir_path = setup_temp_dir();

    // Run "udv init"
    let output = run_udv_init(binary_path, temp_dir_path);

    assert!(
        !output.status.success(),
        "udv init unexpectedly succeeded despite not being run in the root of a git repository"
    );
}
