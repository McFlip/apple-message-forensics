use std::{fs, path::Path};

use apple_message_forensics::{setup_output_dir, write_evidence_hashes};
use tempfile::tempdir;

struct ExpectedHash {
    relative_path: &'static str,
    sha256: &'static str,
}

#[test]
fn writes_sha256_hashes_for_collected_messages_and_address_book_evidence() {
    let temp_dir = tempdir().expect("create temporary test directory");
    let home_dir = temp_dir.path().join("home");
    let output_dir = temp_dir.path().join("output");
    setup_output_dir(&output_dir).expect("setup output directory");

    // Mimics:
    // ~/Library/Messages/
    // ~/Library/Application Support/AddressBook/Sources/<source-id>/
    let messages_dir = home_dir.join("Library").join("Messages");
    let address_book_dir = home_dir
        .join("Library")
        .join("Application Support")
        .join("AddressBook")
        .join("Sources")
        .join("test-address-book-source");

    fs::create_dir_all(&messages_dir).expect("create Messages test directory");
    fs::create_dir_all(&address_book_dir).expect("create AddressBook test directory");

    // Use fixed byte contents with independently known SHA-256 values.
    write_file(&messages_dir.join("chat.db"), b"abc");
    write_file(&messages_dir.join("chat.db-wal"), b"hello");
    write_file(&messages_dir.join("chat.db-shm"), b"test");
    write_file(
        &address_book_dir.join("AddressBook-v22.abcddb"),
        b"hello world",
    );

    let expected_hashes = [
        ExpectedHash {
            relative_path: "Library/Messages/chat.db",
            sha256: "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",
        },
        ExpectedHash {
            relative_path: "Library/Messages/chat.db-wal",
            sha256: "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",
        },
        ExpectedHash {
            relative_path: "Library/Messages/chat.db-shm",
            sha256: "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",
        },
        ExpectedHash {
            relative_path: "Library/Application Support/AddressBook/Sources/test-address-book-source/AddressBook-v22.abcddb",
            sha256: "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",
        },
    ];

    let hashes_path = write_evidence_hashes(&home_dir, &output_dir)
        .expect("write SHA-256 manifest for collected evidence");

    // let hashes_path = output_dir.join("evidence").join("vault").join("hashes.txt");
    assert!(
        hashes_path.is_file(),
        "expected hash manifest at {}",
        hashes_path.display()
    );

    let manifest = fs::read_to_string(&hashes_path).expect("read hashes.txt");

    for expected in expected_hashes {
        let expected_line = format!("{}  {}", expected.sha256, expected.relative_path);

        assert!(
            manifest.lines().any(|line| line == expected_line),
            "hash manifest did not contain expected line:\n{expected_line}\n\nactual manifest:\n{manifest}",
        );
    }
}

fn write_file(path: &Path, contents: &[u8]) {
    fs::write(path, contents).unwrap_or_else(|error| {
        panic!("write test file {}: {error}", path.display());
    });
}
