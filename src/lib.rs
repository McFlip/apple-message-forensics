use rusqlite::Connection;
use std::path::PathBuf;

const REQUIRED_TABLES: [&str; 3] = ["message", "chat", "attachment"];

pub fn check_source(chat_db: PathBuf) -> Result<PathBuf, String> {
    if !chat_db.exists() {
        return Err(format!("source file does not exist: {}", chat_db.display()));
    }

    if !chat_db.is_file() {
        return Err(format!(
            "source path is not a regular file: {}",
            chat_db.display()
        ));
    }

    let connection = Connection::open(&chat_db).map_err(|err| {
        format!(
            "cannot open source database '{}': {}",
            chat_db.display(),
            err
        )
    })?;

    for table in REQUIRED_TABLES {
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
            .map_err(|err| {
                format!(
                    "cannot inspect source database '{}': {}",
                    chat_db.display(),
                    err
                )
            })?;

        if !exists {
            return Err(format!(
                "source database is missing required table '{}': {}",
                table,
                chat_db.display()
            ));
        }
    }

    Ok(chat_db)
}
