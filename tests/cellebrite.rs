use apple_message_forensics::create_cellebrite_output;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use tempfile::tempdir;

fn sha256(contents: &[u8]) -> String {
    hex::encode(Sha256::digest(contents))
}

fn write_evidence_file(
    working_copy: &Path,
    manifest_entries: &mut Vec<String>,
    relative_path: &str,
    contents: &[u8],
) {
    let source_path = working_copy.join(relative_path);
    fs::create_dir_all(source_path.parent().expect("evidence file has a parent"))
        .expect("create evidence directory");
    fs::write(source_path, contents).expect("write evidence file");
    manifest_entries.push(format!("{}  {}", sha256(contents), relative_path));
}

#[test]
fn creates_cellebrite_layout_verifies_files_and_renames_sms_database() {
    let output_dir = tempdir().expect("create output directory");
    let working_copy = output_dir.path().join("evidence/working_copy");
    fs::create_dir_all(&working_copy).expect("create working copy");

    let address_book_path =
        working_copy.join("Library/Application Support/AddressBook/AddressBook-v22.abcddb");
    fs::create_dir_all(address_book_path.parent().expect("database has a parent"))
        .expect("create AddressBook directory");
    fs::copy(
        "tests/fixtures/fakehome/Library/Application Support/AddressBook/AddressBook-v22.abcddb",
        &address_book_path,
    )
    .expect("copy fixture DB");
    let address_book = fs::read(&address_book_path).expect("read mock AddressBook database");

    let evidence = [
        ("Library/Messages/chat.db", b"chat database".as_slice()),
        (
            "Library/Messages/chat.db-wal",
            b"write-ahead log".as_slice(),
        ),
        ("Library/Messages/chat.db-shm", b"shared memory".as_slice()),
        (
            "Library/Messages/Attachments/photo.jpg",
            b"attachment".as_slice(),
        ),
    ];
    let mut manifest_entries = Vec::new();
    for (relative_path, contents) in evidence {
        write_evidence_file(
            &working_copy,
            &mut manifest_entries,
            relative_path,
            contents,
        );
    }
    manifest_entries.push(format!(
        "{}  {}",
        sha256(&address_book),
        "Library/Application Support/AddressBook/AddressBook-v22.abcddb"
    ));
    fs::write(
        working_copy.join("hash_manifest.txt"),
        format!("{}\n", manifest_entries.join("\n")),
    )
    .expect("write hash manifest");

    create_cellebrite_output(output_dir.path()).expect("create Cellebrite filesystem");

    let intermediate_json = output_dir.path().join("json/cellebrite_addresses.json");
    let exported_contacts: serde_json::Value =
        serde_json::from_slice(&fs::read(intermediate_json).expect("read exported contacts"))
            .expect("parse exported contacts");
    assert_eq!(exported_contacts["format"], "apple-addressbook-forensic-v1");
    assert!(
        exported_contacts["contacts"]
            .as_array()
            .expect("contacts array")
            .iter()
            .any(|contact| {
                contact["name"]["first"] == "Alice" && contact["name"]["last"] == "Example"
            })
    );

    let cellebrite_root = output_dir
        .path()
        .join("cellebrite/private/var/mobile/Library");
    assert!(
        !cellebrite_root
            .join("AddressBook/AddressBook-v22.abcddb")
            .exists()
    );
    let output_address_book_path = cellebrite_root.join("AddressBook/AddressBook.sqlitedb");
    assert!(output_address_book_path.is_file());
    let address_book =
        Connection::open(output_address_book_path).expect("open copied AddressBook database");
    for table in [
        "ABPersonSearchKey",
        "ABGroupMembers",
        "ABMultiValue",
        "ABMultiValueLabel",
        "ABMultiValueEntry",
        "ABMultiValueEntryKey",
        "ABPhoneLastFour",
        "ABPersonMultiValueDeletes",
        "ClientCursor",
        "ClientSequence",
        "ClientCursorConsumed",
        "LimitedAccess",
        "ABStore",
        "ABAccount",
        "ABGroup",
        "ABGroupChanges",
        "ABPerson",
        "ABPersonChanges",
        "ABPersonLink",
        "FirstSortSectionCount",
        "LastSortSectionCount",
        "FirstSortSectionCountTotal",
        "LastSortSectionCountTotal",
    ] {
        let exists: bool = address_book
            .query_row(
                "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1)",
                [table],
                |row| row.get(0),
            )
            .expect("query AddressBook table");
        assert!(exists, "expected AddressBook table '{table}' to exist");
    }
    assert_eq!(
        address_book
            .query_row(
                "SELECT First || ' ' || Last FROM ABPerson WHERE ROWID = 11",
                [],
                |row| row.get::<_, String>(0),
            )
            .expect("query mock contact"),
        "Alice Example"
    );
    assert_eq!(
        address_book
            .query_row(
                "SELECT value FROM ABMultiValue WHERE record_id = 11 AND property = 4",
                [],
                |row| row.get::<_, String>(0),
            )
            .expect("query mock contact email"),
        "alice@example.com"
    );
    assert_eq!(
        address_book
            .query_row(
                "SELECT value FROM ABMultiValue WHERE record_id = 1 AND property = 3",
                [],
                |row| row.get::<_, String>(0),
            )
            .expect("query mock contact phone"),
        "+15550000001"
    );

    let sms_dir = cellebrite_root.join("SMS");
    assert_eq!(
        fs::read(sms_dir.join("sms.db")).expect("read renamed Messages database"),
        b"chat database"
    );
    assert_eq!(
        fs::read(sms_dir.join("sms.db-wal")).expect("read renamed WAL file"),
        b"write-ahead log"
    );
    assert_eq!(
        fs::read(sms_dir.join("sms.db-shm")).expect("read renamed SHM file"),
        b"shared memory"
    );
    assert_eq!(
        fs::read(sms_dir.join("Attachments/photo.jpg")).expect("read copied attachment"),
        b"attachment"
    );
    assert!(!sms_dir.join("chat.db").exists());
    assert!(!sms_dir.join("chat.db-wal").exists());
    assert!(!sms_dir.join("chat.db-shm").exists());
}

#[test]
fn rejects_an_output_directory_that_does_not_exist() {
    let temp_dir = tempdir().expect("create temporary directory");
    let missing_output = temp_dir.path().join("missing");

    let error = create_cellebrite_output(&missing_output).expect_err("output must exist");

    assert!(error.contains("output directory does not exist"));
}

#[test]
fn rejects_a_copied_file_missing_from_the_manifest() {
    let output_dir = tempdir().expect("create output directory");
    let working_copy = output_dir.path().join("evidence/working_copy");
    let source_path = working_copy.join("Library/Messages/chat.db");
    fs::create_dir_all(source_path.parent().expect("database has a parent"))
        .expect("create Messages directory");
    fs::write(source_path, b"chat database").expect("write Messages database");
    fs::create_dir_all(working_copy.join("Library/Application Support/AddressBook"))
        .expect("create AddressBook directory");
    fs::write(working_copy.join("hash_manifest.txt"), "").expect("write empty manifest");

    let error =
        create_cellebrite_output(output_dir.path()).expect_err("file must be listed in manifest");

    assert!(error.contains("is missing from hash manifest"));
}

#[test]
fn rejects_a_copied_file_with_a_hash_mismatch() {
    let output_dir = tempdir().expect("create output directory");
    let working_copy = output_dir.path().join("evidence/working_copy");
    let addressbook_dir = working_copy.join("Library/Application Support/AddressBook");
    let messages_dir = working_copy.join("Library/Messages");
    fs::create_dir_all(&addressbook_dir).expect("create AddressBook directory");
    fs::create_dir_all(&messages_dir).expect("create Messages directory");
    fs::write(messages_dir.join("chat.db"), b"chat database").expect("write Messages database");
    fs::write(
        working_copy.join("hash_manifest.txt"),
        format!("{}  Library/Messages/chat.db\n", "0".repeat(64)),
    )
    .expect("write incorrect hash manifest");

    let error = create_cellebrite_output(output_dir.path()).expect_err("hash must match manifest");

    assert!(error.contains("SHA-256 mismatch"));
}
