use apple_message_forensics::archive_evidence;
use std::fs;
use tempfile::tempdir;
use zip::ZipArchive;

/// Tests that archive_evidence correctly archives files from a source directory.
#[test]
fn archives_files_to_archive() {
    let home_temp_dir = tempdir().expect("create temporary home directory");
    let output_temp_dir = tempdir().expect("create temporary output directory");

    // Create test evidence files in the source directory structure
    let messages_dir = home_temp_dir.path().join("Library/Messages");
    fs::create_dir_all(&messages_dir).expect("create Messages directory");

    let chat_db_path = messages_dir.join("chat.db");
    let manifest_path = messages_dir.join("manifest.txt");

    // Create a test chat database file
    fs::write(&chat_db_path, b"test iMessage database content").expect("write test chat.db");
    fs::write(&manifest_path, b"chat.db\n").expect("write test manifest.txt");

    let attachment_path = messages_dir.join("Attachments/image.dat");
    fs::create_dir_all(attachment_path.parent().unwrap()).expect("create attachments directory");
    fs::write(&attachment_path, b"test attachment content").expect("write test attachment");

    // Create the archive path
    let archive_path = output_temp_dir.path().join("evidence.zip");

    // Call the archive_evidence function
    archive_evidence(&messages_dir, &archive_path).expect("create evidence archive");

    // Verify the archive was created
    assert!(archive_path.exists(), "Archive file should exist");

    let archive_file = fs::File::open(&archive_path).expect("open evidence archive");
    let mut archive = ZipArchive::new(archive_file).expect("read evidence ZIP archive");

    assert_eq!(archive.len(), 3, "Archive should contain all source files");
    for (name, expected_contents) in [
        ("chat.db", b"test iMessage database content".as_slice()),
        ("manifest.txt", b"chat.db\n".as_slice()),
        (
            "Attachments/image.dat",
            b"test attachment content".as_slice(),
        ),
    ] {
        let mut file = archive.by_name(name).expect("find file in archive");
        let mut contents = Vec::new();
        std::io::Read::read_to_end(&mut file, &mut contents).expect("read archived file contents");
        assert_eq!(
            contents, expected_contents,
            "Unexpected contents for {name}"
        );
    }
}

/// Tests that archive_evidence handles error cases correctly.
#[test]
fn archives_nonexistent_directory_gives_error() {
    let home_temp_dir = tempdir().expect("create temporary directory");
    let output_temp_dir = tempdir().expect("create temporary output directory");

    // Create a path that doesn't exist
    let archive_path = output_temp_dir.path().join("nonexistent_archive.zip");

    // Verify the archive does not exist yet
    assert!(
        !archive_path.exists(),
        "Archive file should not exist before archiving"
    );

    // Try to archive from a non-existent directory
    let result = archive_evidence(&home_temp_dir.path().join("NonExistentDir"), &archive_path);

    assert!(result.is_err(), "archiving a missing directory should fail");

    // This should return an error (though we expect the first error might be from reading the dir)
    // The exact error behavior is implementation-dependent but should not create a partial archive
}
