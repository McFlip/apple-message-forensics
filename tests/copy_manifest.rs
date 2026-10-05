use std::fs;

use apple_message_forensics::copy_and_verify_manifest_files;
use tempfile::tempdir;

#[test]
fn copies_and_verifies_files_from_hash_manifest() {
    const CONTENTS: &[u8] = b"abc";
    const SHA256: &str = "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad";
    const RELATIVE_PATH: &str = "Library/Messages/chat.db";

    let home_temp_dir = tempdir().expect("create temporary home directory");
    let output_temp_dir = tempdir().expect("create temporary output directory");

    let source_path = home_temp_dir.path().join(RELATIVE_PATH);
    fs::create_dir_all(source_path.parent().expect("source has parent"))
        .expect("create source directory");
    fs::write(&source_path, CONTENTS).expect("write source file");

    let manifest_path = output_temp_dir.path().join("hashes.txt");
    fs::write(&manifest_path, format!("{SHA256}  {RELATIVE_PATH}\n")).expect("write hash manifest");

    copy_and_verify_manifest_files(&manifest_path, home_temp_dir.path(), output_temp_dir.path())
        .expect("copy and verify manifest files");

    let copied_path = output_temp_dir.path().join(RELATIVE_PATH);
    assert_eq!(
        fs::read(&copied_path).expect("read copied file"),
        CONTENTS,
        "copied file should match the source contents",
    );
}
