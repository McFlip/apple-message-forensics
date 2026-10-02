use rusqlite::Connection;
use tempfile::TempDir;

const SCHEMA: &str = include_str!("fixtures/schema.sql");

pub fn create_test_chat_db() -> (TempDir, Connection) {
    let temp_dir = TempDir::new().expect("failed to create temporary directory");
    let db_path = temp_dir.path().join("chat.db");

    let connection = Connection::open(&db_path).expect("failed to create test chat database");

    connection
        .execute_batch(SCHEMA)
        .expect("failed to initialize test database schema");

    (temp_dir, connection)
}
