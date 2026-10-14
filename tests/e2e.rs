use rusqlite::Connection;
use std::fs;
use tempfile::TempDir;

const CHAT_SCHEMA_SQL: &str = include_str!("fixtures/e2e/chat/0.sql");
const CHAT_DATA_SQL: &str = include_str!("fixtures/e2e/chat/1.sql");
const ADDRESS_BOOK_SCHEMA_SQL: &str = include_str!("fixtures/e2e/addressbook/0.sql");
const ADDRESS_BOOK_DATA_SQL: &str = include_str!("fixtures/e2e/addressbook/1.sql");

const ATTACHMENT_FIXTURE_NAMES: [&str; 3] = [
    "exploits_of_a_mom_2x.png",
    "science_montage.png",
    "security.png",
];

fn build_fake_home(home_dir: &std::path::Path) {
    let messages_dir = home_dir.join("Library").join("Messages");
    fs::create_dir_all(&messages_dir).expect("failed to create Library/Messages");

    let chat_db = Connection::open(messages_dir.join("chat.db"))
        .expect("failed to create fake chat database");
    chat_db
        .execute_batch(CHAT_SCHEMA_SQL)
        .expect("failed to apply chat schema fixture");
    chat_db
        .execute_batch(CHAT_DATA_SQL)
        .expect("failed to apply chat data fixture");

    let attachments_dir = messages_dir.join("Attachments");
    fs::create_dir_all(&attachments_dir).expect("failed to create Attachments directory");
    for name in ATTACHMENT_FIXTURE_NAMES {
        let fixture = std::path::PathBuf::from("tests/fixtures/e2e/chat/attachments").join(name);
        let contents = fs::read(&fixture).expect("failed to read attachment fixture");
        fs::write(&attachments_dir.join(name), contents)
            .expect("failed to write attachment fixture");
    }

    let address_book_dir = home_dir
        .join("Library")
        .join("Application Support")
        .join("AddressBook");
    fs::create_dir_all(&address_book_dir).expect("failed to create AddressBook directory");

    let address_book_db = Connection::open(address_book_dir.join("AddressBook-v22.abcddb"))
        .expect("failed to create fake address book database");
    address_book_db
        .execute_batch(ADDRESS_BOOK_SCHEMA_SQL)
        .expect("failed to apply address book schema fixture");
    address_book_db
        .execute_batch(ADDRESS_BOOK_DATA_SQL)
        .expect("failed to apply address book data fixture");
}

#[test]
fn collect_against_fake_home_produces_full_output() {
    let fake_home = TempDir::new().expect("failed to create fake home temporary directory");
    let output = TempDir::new().expect("failed to create output temporary directory");

    build_fake_home(&fake_home.path());

    let result = std::process::Command::new("target/debug/apple-message-forensics")
        .arg("--home")
        .arg(fake_home.path())
        .arg("-o")
        .arg(output.path())
        .arg("collect")
        .output()
        .expect("failed to run collect against fake home");

    if !result.status.success() {
        eprintln!(
            "collect run failed:\nstdout:\n{:?}\nstderr:\n{:?}",
            result.stdout, result.stderr,
        );
    }
    assert!(
        result.status.success(),
        "collect against fake home should succeed"
    );

    let output_dir = output.path();

    assert!(
        output_dir
            .join("evidence")
            .join("vault")
            .join("vault.zip")
            .is_file(),
        "expected evidence vault archive to exist",
    );
    assert!(
        output_dir
            .join("evidence")
            .join("working_copy")
            .join("hash_manifest.txt")
            .is_file(),
        "expected evidence hash manifest to exist",
    );

    let json_dir = output_dir.join("json");
    for name in [
        "handles.json",
        "group_memberships.json",
        "addresses.json",
        "messages.json",
        "joined.json",
    ] {
        assert!(json_dir.join(name).is_file(), "expected {name} to exist");
    }

    let report_dir = output_dir.join("report");
    for name in [
        "index.html",
        "contacts/index.html",
        "chats/index.html",
        "messages/index.html",
        "media/index.html",
    ] {
        assert!(
            report_dir.join(name).is_file(),
            "expected report page {name} to exist"
        );
    }
}
