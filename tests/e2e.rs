use rusqlite::Connection;
use std::fs;
use tempfile::TempDir;

const CHAT_SCHEMA_SQL: &str = include_str!("fixtures/e2e/chat/0.sql");
const CHAT_DATA_SQL: &str = include_str!("fixtures/e2e/chat/1.sql");
const ADDRESS_BOOK_SCHEMA_SQL: &str = include_str!("fixtures/e2e/addressbook/0.sql");
const ADDRESS_BOOK_DATA_SQL: &str = include_str!("fixtures/e2e/addressbook/1.sql");

const ATTACHMENT_FIXTURES: [(&str, &str); 3] = [
    ("exploits_of_a_mom_2x.png", "exploits_of_a_mom_2x.png"),
    ("science_montage.png", "science_montage.png"),
    ("security.png", "1a/2f/GUID-ATT-03/security.png"),
];

const REPORT_JSON_FILES: [&str; 6] = [
    "handles.json",
    "group_memberships.json",
    "addresses.json",
    "messages.json",
    "joined.json",
    "stats.json",
];

const REPORT_PAGES: [&str; 5] = [
    "index.html",
    "contacts/index.html",
    "chats/index.html",
    "messages/index.html",
    "media/index.html",
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
    for (fixture_name, relative_path) in ATTACHMENT_FIXTURES {
        let fixture =
            std::path::PathBuf::from("tests/fixtures/e2e/chat/attachments").join(fixture_name);
        let contents = fs::read(&fixture).expect("failed to read attachment fixture");

        let destination = attachments_dir.join(relative_path);
        fs::create_dir_all(destination.parent().expect("attachment has no parent"))
            .expect("failed to create attachment directory");
        fs::write(&destination, contents).expect("failed to write attachment fixture");
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
fn e2e_collect_against_fake_home_produces_full_output() {
    let fake_home = TempDir::new().expect("failed to create fake home temporary directory");
    let output = TempDir::new().expect("failed to create output temporary directory");

    build_fake_home(fake_home.path());

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
            "collect run failed:\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr),
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
    for name in REPORT_JSON_FILES {
        assert!(json_dir.join(name).is_file(), "expected {name} to exist");
    }

    let handles =
        fs::read_to_string(json_dir.join("handles.json")).expect("failed to read handles json");
    assert!(
        handles.contains("+15550000001"),
        "expected handles json to contain fixture handle +15550000001",
    );

    let statistics =
        fs::read_to_string(json_dir.join("stats.json")).expect("failed to read statistics json");
    let statistics: serde_json::Value =
        serde_json::from_str(&statistics).expect("failed to parse statistics json");
    assert!(
        statistics["total_messages"]
            .as_u64()
            .is_some_and(|count| count > 0),
        "expected statistics to include a positive total message count",
    );
    assert!(
        statistics["total_attachments"]
            .as_u64()
            .is_some_and(|count| count > 0),
        "expected statistics to include a positive total attachment count",
    );
    assert!(
        statistics["top_talkers"]
            .as_array()
            .is_some_and(|talkers| !talkers.is_empty() && talkers.len() <= 10),
        "expected statistics to include up to 10 top talkers",
    );

    let report_dir = output_dir.join("report");
    for name in REPORT_PAGES {
        assert!(
            report_dir.join(name).is_file(),
            "expected report page {name} to exist"
        );
    }

    let homepage =
        fs::read_to_string(report_dir.join("index.html")).expect("failed to read report homepage");
    for label in ["Total Messages:", "Total Attachments:", "Top Talkers"] {
        assert!(
            homepage.contains(label),
            "expected report homepage to display {label}",
        );
    }
    for talker in statistics["top_talkers"]
        .as_array()
        .expect("statistics include top talkers")
    {
        let talker_href = format!("href=\"contacts/contact_{}.html\"", talker["contact_id"]);
        assert!(
            homepage.contains(&talker_href),
            "expected top talker {} to link to its contact page",
            talker["name"]
        );
    }

    for (_, relative_path) in ATTACHMENT_FIXTURES {
        assert!(
            report_dir.join("attachments").join(relative_path).is_file(),
            "expected attachment {relative_path} to be copied into the report",
        );
    }
}
