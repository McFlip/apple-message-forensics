use apple_message_forensics::archive_evidence;
use std::fs;
use tempfile::tempdir;

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

    // Create a text file with evidence
    let notes_path = home_temp_dir.path().join("Library/Notes.txt");
    fs::create_dir_all(notes_path.parent().unwrap()).expect("create Notes directory");
    fs::write(&notes_path, b"Test note content").expect("write test note");

    // Create a subdirectory with additional evidence
    let addressbook_path = home_temp_dir.path().join("Library/Application Support/AddressBook.db");
    fs::create_dir_all(addressbook_path.parent().unwrap()).expect("create AddressBook directory");
    fs::write(&addressbook_path, b"Test AddressBook database content").expect("write test AddressBook.db");

    // Create the archive path
    let archive_path = output_temp_dir.path().join("evidence.zip");

    // Call the archive_evidence function
    archive_evidence(&messages_dir, &archive_path).expect("create evidence archive");

    // Verify the archive was created
    assert!(archive_path.exists(), "Archive file should exist");
    
    // Check that the manifest has the expected content
    let contents = fs::read_to_string(&archive_path).expect("read archive contents");
    
    assert!(contents.contains("chat.db"), "Archive should contain reference to chat.db");
    assert!(contents.contains("manifest.txt"), "Archive should contain reference to manifest.txt");

    // Verify the files from source_dir were archived (as expected structure)
    // The archive should contain all discovered evidence files with their relative paths
    
    println!("Archive created at: {:?}", archive_path);
    println!("Archive contents:\n{}", contents);
}

/// Tests that archive_evidence handles error cases correctly.
#[test]
fn archives_nonexistent_directory_gives_error() {
    let home_temp_dir = tempdir().expect("create temporary directory");
    let output_temp_dir = tempdir().expect("create temporary output directory");

    // Create a path that doesn't exist
    let archive_path = output_temp_dir.path().join("nonexistent_archive.zip");

    // Verify the archive does not exist yet
    assert!(!archive_path.exists(), "Archive file should not exist before archiving");

    // Try to archive from a non-existent directory
    let result = archive_evidence(
        &home_temp_dir.path().join("NonExistentDir"),
        &archive_path,
    );

    assert!(result.is_err(), "archiving a missing directory should fail");

    // This should return an error (though we expect the first error might be from reading the dir)
    // The exact error behavior is implementation-dependent but should not create a partial archive
}
