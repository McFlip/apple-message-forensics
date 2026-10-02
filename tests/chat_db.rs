mod common;

use apple_message_forensics::check_source;
use common::create_test_chat_db;

#[test]
fn creates_chat_database_from_schema() {
    let (_temp_dir, connection) = create_test_chat_db();

    for table in ["message", "chat", "attachment"] {
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(
                    SELECT 1
                    FROM sqlite_master
                    WHERE type = 'table' AND name = ?1
                )",
                [table],
                |row| row.get(0),
            )
            .expect("failed to query sqlite_master");

        assert!(exists, "expected table '{}' to exist", table);
    }
}

#[test]
fn check_source_accepts_test_chat_db() {
    let (temp_dir, _connection) = create_test_chat_db();
    let db_path = temp_dir.path().join("chat.db");

    let result = check_source(db_path.clone());

    assert_eq!(result.unwrap(), db_path);
}
