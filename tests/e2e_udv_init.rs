use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn test_udv_init() {
    let temp_dir = tempdir().unwrap();
    let temp_dir_path = temp_dir.path().to_str().unwrap();

    let binary_path = std::env::current_dir().unwrap().join("target/debug/udv");
    assert!(
        binary_path.exists(),
        "Binary does not exist: {:?}",
        binary_path
    );

    std::env::set_current_dir(&temp_dir_path).expect("Failed to change to temp directory");

    Command::new("git")
        .args(["init", temp_dir_path])
        .output()
        .expect("Failed to initialize git repository for testing");

    // Run "udv init"
    let output = Command::new(binary_path)
        .arg("init")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success(), "Command did not run successfully");

    // Check existence of .udv directory
    let udv_dir = temp_dir.path().join(".udv");
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
