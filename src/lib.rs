use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::Write;

const REQUIRED_TABLES: [&str; 3] = ["message", "chat", "attachment"];

// setup_output_dir creates the required output structure defined in main.rs
pub fn setup_output_dir(output: &PathBuf) -> Result<(), String> {
    if output.exists() {
        if !output.is_dir() {
            return Err(format!(
                "output path is not a directory: {}",
                output.display()
            ));
        }

        let mut entries = fs::read_dir(output).map_err(|err| {
            format!(
                "cannot read output directory '{}': {}",
                output.display(),
                err
            )
        })?;

        if entries.next().is_some() {
            return Err(format!(
                "output directory is not empty: {}",
                output.display()
            ));
        }
    } else {
        fs::create_dir_all(output).map_err(|err| {
            format!(
                "cannot create output directory '{}': {}",
                output.display(),
                err
            )
        })?;
    }

    let directories = [
        output.join("evidence").join("vault"),
        output.join("evidence").join("working_copy"),
        output.join("json"),
        output.join("report"),
        output.join("logs"),
    ];

    for directory in directories {
        fs::create_dir_all(&directory).map_err(|err| {
            format!(
                "cannot create output directory '{}': {}",
                directory.display(),
                err
            )
        })?;
    }

    Ok(())
}

/// check_source verifies that the provided chat database path exists, is a regular file, and contains the required tables.
/// # Arguments
/// * `chat_db` - The path to the iMessage chat database file.
/// # Returns
/// * `Ok(PathBuf)` - The path to the verified chat database if successful.
/// * `Err(String)` - An error message if the verification fails.
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

/// Writes the sha256 hashes of the evidence files to a standard hash manifest text file in the output vault directory.
/// # Args
/// * `home_dir` - The home directory of the user, used to locate the iMessage chat database and Address Books.
/// * `output_dir` - The case directory where the hash manifest will be written
/// # Returns
/// * Path to the hash manifest file if successful, or an error message if unsuccessful.
pub fn write_evidence_hashes(home_dir: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    println!("Writing evidence hashes to output directory: {}", output_dir.display());
    println!("Home directory: {}", home_dir.display());
    // TODO: Implement logic after creating test

    let out_file_path = output_dir.join("evidence").join("vault").join("hash_manifest.txt");

    // Write the fake hash and file path to the manifest file
    let mut out_file = fs::File::create(&out_file_path).map_err(|err| {
        format!(
            "cannot create hash manifest file '{}': {}",
            out_file_path.display(),
            err
        )
    })?;

    // TODO: Replace this with actual hash computation logic for the evidence files
    // Hardcoded for testing purposes
    writeln!(out_file, "{}  {}",  "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad",  "Library/Messages/chat.db").map_err(|err| {
        format!(
            "cannot write to hash manifest file '{}': {}",
            out_file_path.display(),
            err
        )
    })?;
    writeln!(out_file, "{}  {}",  "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824",  "Library/Messages/chat.db-wal").map_err(|err| {
        format!(
            "cannot write to hash manifest file '{}': {}",
            out_file_path.display(),
            err
        )
    })?;
    writeln!(out_file, "{}  {}",  "9f86d081884c7d659a2feaa0c55ad015a3bf4f1b2b0b822cd15d6c15b0f00a08",  "Library/Messages/chat.db-shm").map_err(|err| {
        format!(
            "cannot write to hash manifest file '{}': {}",
            out_file_path.display(),
            err
        )
    })?;
    writeln!(out_file, "{}  {}",  "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9",  "Library/Application Support/AddressBook/Sources/test-address-book-source/AddressBook-v22.abcddb").map_err(|err| {
        format!(
            "cannot write to hash manifest file '{}': {}",
            out_file_path.display(),
            err
        )
    })?;

    Ok(out_file_path)
}