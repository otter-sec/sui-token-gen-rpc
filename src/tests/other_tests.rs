use crate::utils::helpers::sanitize_repo_name;

/// Test to verify that a valid repository name remains unaltered
/// after sanitization.
///
/// A valid repository name contains only alphanumeric characters,
/// underscores, or dashes, and does not include invalid path traversal characters.
#[test]
fn test_safe_path_valid() {
    let valid_target = "sui-token";
    // Assert that the valid target path is not altered after sanitization
    assert_eq!(sanitize_repo_name(valid_target), valid_target);
}

/// Test to ensure that an invalid repository name is sanitized correctly.
///
/// An invalid repository name containing path traversal components
/// (e.g., "../") is transformed into a safe version without special characters.
#[test]
fn test_safe_path_invalid() {
    let invalid_target = "../etc/psswd";
    // Assert that the invalid target path is sanitized by removing path traversal components
    assert_eq!(sanitize_repo_name(invalid_target), "etcpsswd");
}
