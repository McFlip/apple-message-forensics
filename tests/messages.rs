use apple_message_forensics::get_all_messages;
use rusqlite::Connection;
use serde_json::json;
use tempfile::tempdir;

const CHAT_SCHEMA: &str = include_str!("../schema/chat-schema.sql");

#[test]
fn get_all_messages_returns_messages_as_json() {
    let temp_dir = tempdir().expect("create temporary directory");
    let db_path = temp_dir.path().join("chat.db");
    let connection = Connection::open(&db_path).expect("create test chat database");

    let schema = CHAT_SCHEMA
        .lines()
        .take_while(|line| !line.starts_with("CREATE TRIGGER"))
        .filter(|line| *line != "CREATE TABLE sqlite_sequence(name,seq);")
        .collect::<Vec<_>>()
        .join("\n");
    connection
        .execute_batch(&schema)
        .expect("initialize chat database schema");

    for id in 1..=3 {
        connection
            .execute(
                "INSERT INTO chat (ROWID, guid, chat_identifier, display_name)
                 VALUES (?1, ?2, ?3, ?4)",
                (
                    id,
                    format!("chat-guid-{id}"),
                    format!("chat-{id}@example.test"),
                    format!("Chat {id}"),
                ),
            )
            .expect("insert chat");
        connection
            .execute(
                "INSERT INTO handle (ROWID, id, service)
                 VALUES (?1, ?2, 'iMessage')",
                (id, format!("sender-{id}@example.test")),
            )
            .expect("insert handle");
        connection
            .execute(
                "INSERT INTO message (ROWID, guid, text, handle_id, date)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                (
                    id,
                    format!("message-guid-{id}"),
                    format!("Test message {id}"),
                    id,
                    i64::from(id) * 1_000_000_000,
                ),
            )
            .expect("insert message");
        connection
            .execute(
                "INSERT INTO chat_message_join (chat_id, message_id)
                 VALUES (?1, ?2)",
                (id, id),
            )
            .expect("link message to chat");
        connection
            .execute(
                "INSERT INTO attachment (ROWID, guid, filename, original_guid)
                 VALUES (?1, ?2, ?3, ?4)",
                (
                    id,
                    format!("attachment-guid-{id}"),
                    format!("attachment-{id}.txt"),
                    format!("original-attachment-guid-{id}"),
                ),
            )
            .expect("insert attachment");
        connection
            .execute(
                "INSERT INTO message_attachment_join (message_id, attachment_id)
                 VALUES (?1, ?2)",
                (id, id),
            )
            .expect("link attachment to message");
    }

    let actual = get_all_messages(&db_path).expect("get all messages");
    let actual_json: serde_json::Value =
        serde_json::from_str(&actual).expect("returned string should be valid JSON");

    assert_eq!(
        actual_json,
        json!([
            {
                "timestamp": "2001-01-01 00:00:01",
                "chat": "Chat 1",
                "sender": "sender-1@example.test",
                "message": "Test message 1",
                "attachment": "attachment-1.txt"
            },
            {
                "timestamp": "2001-01-01 00:00:02",
                "chat": "Chat 2",
                "sender": "sender-2@example.test",
                "message": "Test message 2",
                "attachment": "attachment-2.txt"
            },
            {
                "timestamp": "2001-01-01 00:00:03",
                "chat": "Chat 3",
                "sender": "sender-3@example.test",
                "message": "Test message 3",
                "attachment": "attachment-3.txt"
            }
        ])
    );
}
