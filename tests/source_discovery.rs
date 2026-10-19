use apple_message_forensics::get_icloud_addresses;
use apple_message_forensics::source_discovery::find_icloud_address_book_uuid;
use rusqlite::Connection;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use tempfile::TempDir;

const CONTAINERS_FIXTURE: &str = include_str!("fixtures/contact-containers.json");

fn fixture_with_source_path(temp_dir: &TempDir) -> (std::path::PathBuf, Value) {
    let mut fixture: Value = serde_json::from_str(CONTAINERS_FIXTURE).unwrap();
    let candidate = fixture["containers"]
        .as_array_mut()
        .unwrap()
        .iter_mut()
        .find(|container| container["isICloudCandidate"] == true)
        .unwrap();
    let source_path = temp_dir.path().join("address-book-source");
    std::fs::create_dir_all(&source_path).unwrap();
    candidate["sourcePath"] = Value::String(source_path.to_string_lossy().into_owned());
    (source_path, fixture)
}

fn write_fixture(path: &Path, fixture: &Value) {
    std::fs::write(path, serde_json::to_vec_pretty(fixture).unwrap()).unwrap();
}

#[test]
fn finds_uuid_from_fixture_and_checks_database_integrity() {
    let temp_dir = TempDir::new().unwrap();
    let (source_path, fixture) = fixture_with_source_path(&temp_dir);
    let database_path = source_path.join("AddressBook-v22.abcddb");
    let connection = Connection::open(&database_path).unwrap();
    connection
        .execute_batch("CREATE TABLE contacts (id INTEGER PRIMARY KEY);")
        .unwrap();
    drop(connection);

    let json_path = temp_dir.path().join("contact-containers.json");
    write_fixture(&json_path, &fixture);

    assert_eq!(
        find_icloud_address_book_uuid(&json_path).unwrap(),
        "01234567-89AB-CDEF-0123-456789ABCDEF"
    );
}

#[test]
fn reports_when_no_icloud_candidate_exists() {
    let temp_dir = TempDir::new().unwrap();
    let mut fixture: Value = serde_json::from_str(CONTAINERS_FIXTURE).unwrap();
    fixture["containers"][0]["isICloudCandidate"] = Value::Bool(false);
    let json_path = temp_dir.path().join("contact-containers.json");
    write_fixture(&json_path, &fixture);

    let error = find_icloud_address_book_uuid(&json_path).unwrap_err();
    assert!(error.contains("sign in to iCloud and sync from the target phone"));
}

#[test]
fn reports_when_multiple_icloud_candidates_exist() {
    let temp_dir = TempDir::new().unwrap();
    let mut fixture: Value = serde_json::from_str(CONTAINERS_FIXTURE).unwrap();
    let duplicate = fixture["containers"][0].clone();
    fixture["containers"]
        .as_array_mut()
        .unwrap()
        .push(duplicate);
    let json_path = temp_dir.path().join("contact-containers.json");
    write_fixture(&json_path, &fixture);

    assert_eq!(
        find_icloud_address_book_uuid(&json_path).unwrap_err(),
        "More than one iCloud Address Book container was found."
    );
}

#[test]
fn reports_when_icloud_database_is_missing() {
    let temp_dir = TempDir::new().unwrap();
    let (_, fixture) = fixture_with_source_path(&temp_dir);
    let json_path = temp_dir.path().join("contact-containers.json");
    write_fixture(&json_path, &fixture);

    let error = find_icloud_address_book_uuid(&json_path).unwrap_err();
    assert!(error.contains("iCloud Address Book database does not exist"));
}

#[test]
fn parses_only_the_address_book_matching_the_icloud_uuid() {
    let temp_dir = TempDir::new().unwrap();
    let working_copy = temp_dir.path().join("working_copy");
    let uuid = "01234567-89AB-CDEF-0123-456789ABCDEF";
    let other_uuid = "FEDCBA98-7654-3210-FEDC-BA9876543210";
    let fixture_db = Path::new(
        "tests/fixtures/fakehome/Library/Application Support/AddressBook/AddressBook-v22.abcddb",
    );
    let selected_path = working_copy
        .join("Library/Application Support/AddressBook/Sources")
        .join(uuid)
        .join("AddressBook-v22.abcddb");
    let other_path = working_copy
        .join("Library/Application Support/AddressBook/Sources")
        .join(other_uuid)
        .join("AddressBook-v22.abcddb");
    fs::create_dir_all(selected_path.parent().unwrap()).unwrap();
    fs::create_dir_all(other_path.parent().unwrap()).unwrap();
    fs::copy(fixture_db, &selected_path).unwrap();
    fs::write(&other_path, b"not a SQLite database").unwrap();

    let manifest_path = working_copy.join("hash_manifest.txt");
    let mut entries = Vec::new();
    for path in [&selected_path, &other_path] {
        let contents = fs::read(path).unwrap();
        let relative_path = path.strip_prefix(&working_copy).unwrap();
        entries.push(format!(
            "{}  {}",
            hex::encode(Sha256::digest(contents)),
            relative_path.display()
        ));
    }
    fs::write(&manifest_path, entries.join("\n")).unwrap();

    let addresses_json = get_icloud_addresses(&manifest_path, &working_copy, uuid).unwrap();
    let addresses: Value = serde_json::from_str(&addresses_json).unwrap();

    assert!(!addresses.as_array().unwrap().is_empty());
}
