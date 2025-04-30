use crate::utils::helpers::sanitize_repo_name;

/// Test to verify that a valid repository name remains unaltered
#[test]
fn test_safe_path_valid() {
    let valid_target = "sui-token";
    assert_eq!(sanitize_repo_name(valid_target), valid_target);
}

/// Test to ensure that an invalid repository name with path traversal is sanitized correctly
#[test]
fn test_safe_path_invalid() {
    let invalid_target = "../etc/psswd";
    assert_eq!(sanitize_repo_name(invalid_target), "etcpsswd");
}

/// Test that removes multiple instances of path traversal sequences
#[test]
fn test_safe_path_multiple_traversals() {
    let invalid_target = "../../var/log";
    assert_eq!(sanitize_repo_name(invalid_target), "varlog");
}

/// Test that removes mixed path traversal patterns
#[test]
fn test_safe_path_mixed_traversals() {
    let invalid_target = "..//..\\config.yaml";
    assert_eq!(sanitize_repo_name(invalid_target), "configyaml");
}

/// Test that removes leading or trailing slashes
#[test]
fn test_safe_path_leading_trailing_slashes() {
    let invalid_target = "/sui-repo/";
    assert_eq!(sanitize_repo_name(invalid_target), "sui-repo");
}

/// Test that removes Windows-style path traversal sequences
#[test]
fn test_safe_path_windows_style() {
    let invalid_target = "..\\..\\Windows\\System32";
    assert_eq!(sanitize_repo_name(invalid_target), "WindowsSystem32");
}

/// Test that removes hidden files (Unix-style dot files)
#[test]
fn test_safe_path_hidden_file() {
    let invalid_target = ".git";
    assert_eq!(sanitize_repo_name(invalid_target), "git");
}

/// Test that sanitizes an empty string
#[test]
fn test_safe_path_empty() {
    let invalid_target = "";
    assert_eq!(sanitize_repo_name(invalid_target), "default_repo"); // Use a fallback name if needed
}

/// Test that removes special characters
#[test]
fn test_safe_path_special_chars() {
    let invalid_target = "sui$%&repo@!";
    assert_eq!(sanitize_repo_name(invalid_target), "sui$%&repo@!");
}

/// Test that removes dots used to try to reconstruct ".."
#[test]
fn test_safe_path_dot_reconstruction() {
    let invalid_target = "./.";
    assert_eq!(sanitize_repo_name(invalid_target), "default_repo");
}

/// Test that removes mixed separators and special characters
#[test]
fn test_safe_path_mixed_chars() {
    let invalid_target = "/.././repo@#!/";
    assert_eq!(sanitize_repo_name(invalid_target), "repo@#!");
}
