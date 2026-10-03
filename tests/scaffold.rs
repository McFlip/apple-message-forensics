mod common;

use std::fs;
use tempfile::tempdir;

use apple_message_forensics::setup_output_dir;

#[test]
fn setup_output_dir_creates_missing_directory() {
    let temp = tempdir().expect("failed to create temporary directory");
    let output = temp.path().join("output");

    assert!(!output.exists());

    setup_output_dir(&output).unwrap();

    assert!(output.is_dir());
    assert!(output.join("evidence").join("vault").is_dir());
    assert!(output.join("evidence").join("working_copy").is_dir());
    assert!(output.join("json").is_dir());
    assert!(output.join("report").is_dir());
    assert!(output.join("logs").is_dir());
}

#[test]
fn setup_output_dir_accepts_existing_empty_directory() {
    let temp = tempdir().expect("failed to create temporary directory");
    let output = temp.path().join("output");

    fs::create_dir(&output).expect("failed to create output directory");

    assert!(output.is_dir());
    assert_eq!(
        fs::read_dir(&output)
            .expect("failed to read output directory")
            .count(),
        0
    );

    setup_output_dir(&output).unwrap();

    assert!(output.join("evidence").join("vault").is_dir());
    assert!(output.join("evidence").join("working_copy").is_dir());
    assert!(output.join("json").is_dir());
    assert!(output.join("report").is_dir());
    assert!(output.join("logs").is_dir());
}

#[test]
fn setup_output_dir_rejects_non_empty_directory() {
    let temp = tempdir().expect("failed to create temporary directory");
    let output = temp.path().join("output");

    fs::create_dir(&output).expect("failed to create output directory");
    fs::write(output.join("existing.txt"), "test").expect("failed to create existing file");

    let result = setup_output_dir(&output);

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        format!("output directory is not empty: {}", output.display())
    );
}

#[test]
fn setup_output_dir_rejects_file_as_output_path() {
    let temp = tempdir().expect("failed to create temporary directory");
    let output = temp.path().join("output");

    fs::write(&output, "test").expect("failed to create test file");

    let result = setup_output_dir(&output);

    assert!(result.is_err());
    assert_eq!(
        result.unwrap_err(),
        format!("output path is not a directory: {}", output.display())
    );
}
