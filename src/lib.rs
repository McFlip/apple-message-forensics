use chrono::{DateTime, Utc};
use hex;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::os::unix::fs::PermissionsExt;
use std::{
    collections::HashMap,
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    thread,
    time::Duration,
};

const MAX_COPY_ATTEMPTS: usize = 3;
const HASH_RETRY_DELAY: Duration = Duration::from_secs(60);
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

/// Calculates the SHA-256 hash for a given file.
fn calculate_sha256(path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("cannot open file '{}': {}", path.display(), e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];

    loop {
        match file.read(&mut buffer) {
            Ok(0) => break,
            Ok(count) => hasher.update(&buffer[..count]),
            Err(e) => return Err(format!("error reading file '{}': {}", path.display(), e)),
        }
    }
    let hash = hasher.finalize();
    Ok(hex::encode(hash))
}

/// Recursively finds regular files beneath the Messages and AddressBook
/// evidence directories and returns paths relative to `home_dir`.
fn find_evidence_files(home_dir: &Path) -> Result<Vec<PathBuf>, String> {
    fn visit_directory(
        directory: &Path,
        home_dir: &Path,
        evidence_files: &mut Vec<PathBuf>,
    ) -> Result<(), String> {
        let entries = fs::read_dir(directory).map_err(|err| {
            format!(
                "cannot read evidence directory '{}': {}",
                directory.display(),
                err
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|err| {
                format!(
                    "cannot read an entry in evidence directory '{}': {}",
                    directory.display(),
                    err
                )
            })?;

            let path = entry.path();
            let file_type = entry.file_type().map_err(|err| {
                format!("cannot inspect evidence path '{}': {}", path.display(), err)
            })?;

            if file_type.is_dir() {
                visit_directory(&path, home_dir, evidence_files)?;
            } else if file_type.is_file() {
                let relative_path = path.strip_prefix(home_dir).map_err(|err| {
                    format!(
                        "evidence path '{}' is not beneath home directory '{}': {}",
                        path.display(),
                        home_dir.display(),
                        err
                    )
                })?;

                evidence_files.push(relative_path.to_path_buf());
            }
        }

        Ok(())
    }

    let evidence_roots = [
        home_dir.join("Library").join("Messages"),
        home_dir
            .join("Library")
            .join("Application Support")
            .join("AddressBook"),
    ];

    let mut evidence_files = Vec::new();

    for evidence_root in evidence_roots {
        visit_directory(&evidence_root, home_dir, &mut evidence_files)?;
    }

    evidence_files.sort();
    Ok(evidence_files)
}

/// Writes the sha256 hashes of the evidence files to a standard hash manifest text file in the output vault directory.
/// # Args
/// * `home_dir` - The home directory of the user, used to locate the iMessage chat database and Address Books.
/// * `output_dir` - The case directory where the hash manifest will be written
/// # Returns
/// * Path to the hash manifest file if successful, or an error message if unsuccessful.
pub fn write_evidence_hashes(home_dir: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    let out_file_path = output_dir
        .join("evidence")
        .join("working_copy")
        .join("hash_manifest.txt");
    let mut manifest_entries: Vec<(String, String)> = Vec::new();

    // recursively scan the home_dir and get relative paths
    let evidence_files = find_evidence_files(home_dir)?;

    println!(
        "Writing evidence hashes to output directory: {}",
        out_file_path.display()
    );
    println!("Home directory: {}", home_dir.display());

    for relative_path in evidence_files.iter() {
        let absolute_path = home_dir.join(relative_path);

        match calculate_sha256(&absolute_path) {
            Ok(hash) => {
                manifest_entries.push((hash, relative_path.display().to_string()));
            }
            Err(e) => {
                eprintln!(
                    "Warning: Could not hash file {}: {}",
                    relative_path.display(),
                    e
                );
            }
        }
    }

    // Write the manifest file
    let mut out_file = fs::File::create(&out_file_path).map_err(|err| {
        format!(
            "cannot create hash manifest file '{}': {}",
            out_file_path.display(),
            err
        )
    })?;

    for (hash, path) in manifest_entries {
        writeln!(out_file, "{}  {}", hash, path).map_err(|err| {
            format!(
                "cannot write to hash manifest file '{}': {}",
                out_file_path.display(),
                err
            )
        })?;
    }
    set_read_only(&out_file_path).map_err(|err| {
        format!(
            "cannot set read-only permissions on manifest file {}: {}",
            out_file_path.display(),
            err
        )
    })?;

    Ok(out_file_path)
}

/// Copies every file listed in a SHA-256 manifest to `output_dir`.
///
/// Manifest entries must use the format:
///
/// ```text
/// <sha256>  <relative path>
/// ```
///
/// Source files are resolved relative to the manifest's parent directory.
/// Destination paths are created beneath `output_dir`.
///
/// A copied file is hashed after each copy. A mismatch is retried after one
/// minute, for a maximum of three total attempts per file.
pub fn copy_and_verify_manifest_files(
    manifest_path: &Path,
    home_dir: &Path,
    output_dir: &Path,
) -> Result<(), String> {
    let manifest_contents = fs::read_to_string(manifest_path).map_err(|err| {
        format!(
            "cannot read hash manifest '{}': {}",
            manifest_path.display(),
            err
        )
    })?;

    for (line_number, line) in manifest_contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let (expected_hash, relative_path) =
            parse_hash_manifest_line(line, line_number + 1, manifest_path)?;

        let source_path = home_dir.join(&relative_path);
        let destination_path = output_dir.join(&relative_path);

        let destination_parent = destination_path.parent().ok_or_else(|| {
            format!(
                "cannot determine destination directory for '{}'",
                destination_path.display()
            )
        })?;

        fs::create_dir_all(destination_parent).map_err(|err| {
            format!(
                "cannot create evidence directory '{}': {}",
                destination_parent.display(),
                err
            )
        })?;

        for attempt in 1..=MAX_COPY_ATTEMPTS {
            fs::copy(&source_path, &destination_path).map_err(|err| {
                format!(
                    "cannot copy evidence file '{}' to '{}': {}",
                    source_path.display(),
                    destination_path.display(),
                    err
                )
            })?;

            let actual_hash = calculate_sha256(&destination_path)?;

            if actual_hash.eq_ignore_ascii_case(&expected_hash) {
                set_read_only(&destination_path).map_err(|err| {
                    format!(
                        "cannot set read-only permissions on copied evidence file {}: {}",
                        destination_path.display(),
                        err
                    )
                })?;
                break;
            }

            if attempt == MAX_COPY_ATTEMPTS {
                return Err(format!(
                    "SHA-256 mismatch after {} attempts for '{}': expected {}, got {}",
                    MAX_COPY_ATTEMPTS,
                    relative_path.display(),
                    expected_hash,
                    actual_hash
                ));
            }

            thread::sleep(HASH_RETRY_DELAY);
        }
    }

    Ok(())
}

fn parse_hash_manifest_line(
    line: &str,
    line_number: usize,
    manifest_path: &Path,
) -> Result<(String, PathBuf), String> {
    if line.len() < 67 {
        return Err(format!(
            "invalid manifest entry on line {} in '{}': expected '<sha256>  <relative path>'",
            line_number,
            manifest_path.display()
        ));
    }

    let (expected_hash, remainder) = line.split_at(64);

    if !expected_hash
        .chars()
        .all(|character| character.is_ascii_hexdigit())
    {
        return Err(format!(
            "invalid SHA-256 value on line {} in '{}'",
            line_number,
            manifest_path.display()
        ));
    }

    // Accept standard sha256sum text-mode and binary-mode separators:
    // "<hash>  <path>" or "<hash> *<path>".
    let path_text = remainder
        .strip_prefix("  ")
        .or_else(|| remainder.strip_prefix(" *"))
        .ok_or_else(|| {
            format!(
                "invalid manifest separator on line {} in '{}'",
                line_number,
                manifest_path.display()
            )
        })?;

    if path_text.is_empty() {
        return Err(format!(
            "missing file path on line {} in '{}'",
            line_number,
            manifest_path.display()
        ));
    }

    let relative_path = PathBuf::from(path_text);

    if relative_path.is_absolute()
        || relative_path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "manifest path '{}' on line {} in '{}' must be relative and must not contain '..'",
            relative_path.display(),
            line_number,
            manifest_path.display()
        ));
    }

    Ok((expected_hash.to_ascii_lowercase(), relative_path))
}

fn set_read_only(path: &Path) -> Result<(), std::io::Error> {
    let mut permissions = fs::metadata(path)?.permissions();

    // Remove all write bits, preserving read/execute bits.
    permissions.set_mode(permissions.mode() & !0o131313);

    fs::set_permissions(path, permissions)
}

/// Archives all regular files from a source directory to a deflated ZIP file.
/// Files retain their paths relative to `source_dir`.
/// # Arguments
/// * `source_dir` - The directory containing evidence files to be archived.
/// * `archive_path` - The path where the evidence archive will be written.
/// # Returns
/// * `Ok(())` if the archive was created successfully.
/// * `Err(String)` - An error message if archiving fails.
pub fn archive_evidence(source_dir: &Path, archive_path: &Path) -> Result<(), String> {
    let archive_parent = archive_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    fs::create_dir_all(archive_parent).map_err(|err| {
        format!(
            "cannot create parent directory for archive '{}': {}",
            archive_path.display(),
            err
        )
    })?;

    fn collect_files(
        directory: &Path,
        base_dir: &Path,
        entries: &mut Vec<(PathBuf, PathBuf)>,
    ) -> Result<(), String> {
        let directory_entries = fs::read_dir(directory).map_err(|err| {
            format!(
                "cannot read source directory '{}': {}",
                directory.display(),
                err
            )
        })?;

        for entry in directory_entries {
            let entry = entry.map_err(|err| {
                format!(
                    "cannot read an entry in source directory '{}': {}",
                    directory.display(),
                    err
                )
            })?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|err| {
                format!("cannot inspect source path '{}': {}", path.display(), err)
            })?;

            if file_type.is_dir() {
                collect_files(&path, base_dir, entries)?;
            } else if file_type.is_file() {
                let relative_path = path.strip_prefix(base_dir).map_err(|err| {
                    format!(
                        "source file '{}' is not beneath source directory '{}': {}",
                        path.display(),
                        base_dir.display(),
                        err
                    )
                })?;
                entries.push((relative_path.to_path_buf(), path));
            }
        }

        Ok(())
    }

    let mut entries = Vec::new();
    collect_files(source_dir, source_dir, &mut entries)?;
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let archive_file = fs::File::create(archive_path).map_err(|err| {
        format!(
            "cannot create archive file '{}': {}",
            archive_path.display(),
            err
        )
    })?;
    let mut archive = zip::ZipWriter::new(archive_file);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for (relative_path, source_path) in entries {
        let archive_name = relative_path.to_string_lossy().replace('\\', "/");
        archive.start_file(&archive_name, options).map_err(|err| {
            format!(
                "cannot add '{}' to archive '{}': {}",
                source_path.display(),
                archive_path.display(),
                err
            )
        })?;

        let mut source_file = fs::File::open(&source_path).map_err(|err| {
            format!(
                "cannot open source file '{}': {}",
                source_path.display(),
                err
            )
        })?;
        std::io::copy(&mut source_file, &mut archive).map_err(|err| {
            format!(
                "cannot write source file '{}' to archive '{}': {}",
                source_path.display(),
                archive_path.display(),
                err
            )
        })?;
    }

    archive.finish().map_err(|err| {
        format!(
            "cannot finish archive file '{}': {}",
            archive_path.display(),
            err
        )
    })?;

    set_read_only(&archive_path).expect("setting vault ZIP to read-only");

    Ok(())
}

/// Gets all handles from chat.db
/// # Arguments
/// * `chat_db` - The path to the iMessage chat database file.
/// # Returns
/// * `Ok(String)` - Handles in JSON format if successful.
/// * `Err(String)` - An error message if the operation fails.
pub fn get_all_handles(chat_db: &Path) -> Result<String, String> {
    #[derive(serde::Serialize)]
    struct Handle {
        handle_id: i64,
        handle_identifier: String,
        service: String,
        country: Option<String>,
        uncanonicalized_id: Option<String>,
        person_centric_id: Option<String>,
    }

    let connection = Connection::open(chat_db)
        .map_err(|err| format!("cannot open chat database '{}': {}", chat_db.display(), err))?;
    let mut statement = connection
        .prepare(include_str!("../query/handle-query.sql"))
        .map_err(|err| format!("cannot prepare handle query: {}", err))?;
    let handles = statement
        .query_map([], |row| {
            Ok(Handle {
                handle_id: row.get(0)?,
                handle_identifier: row.get(1)?,
                service: row.get(2)?,
                country: row.get(3)?,
                uncanonicalized_id: row.get(4)?,
                person_centric_id: row.get(5)?,
            })
        })
        .map_err(|err| format!("cannot query handles: {}", err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("cannot read handle row: {}", err))?;

    serde_json::to_string(&handles)
        .map_err(|err| format!("cannot serialize handles as JSON: {}", err))
}

/// Gets all addresses from AddressBook-v22.abcddb
/// # Arguments
/// * `address_book` - The path to the AddressBook-v22.abcddb file
/// # Returns
/// * `Ok(String)` - Addresses in JSON format if successful.
/// * `Err(String)` - An error message if the operation fails.
fn get_addresses_from_addressbook(address_book: &Path) -> Result<String, String> {
    #[derive(serde::Serialize)]
    struct Address {
        contact_id: i64,
        contact_unique_id: Option<String>,
        record_type: Option<i64>,
        first_name: Option<String>,
        middle_name: Option<String>,
        last_name: Option<String>,
        nickname: Option<String>,
        organization: Option<String>,
        job_title: Option<String>,
        phone_numbers: Option<String>,
        email_addresses: Option<String>,
    }

    let connection = Connection::open(address_book).map_err(|err| {
        format!(
            "cannot open address book database '{}': {}",
            address_book.display(),
            err
        )
    })?;
    let mut statement = connection
        .prepare(include_str!("../query/addr-query.sql"))
        .map_err(|err| format!("cannot prepare address book query: {}", err))?;
    let addresses = statement
        .query_map([], |row| {
            Ok(Address {
                contact_id: row.get(0)?,
                contact_unique_id: row.get(1)?,
                record_type: row.get(2)?,
                first_name: row.get(3)?,
                middle_name: row.get(4)?,
                last_name: row.get(5)?,
                nickname: row.get(6)?,
                organization: row.get(7)?,
                job_title: row.get(8)?,
                phone_numbers: row.get(9)?,
                email_addresses: row.get(10)?,
            })
        })
        .map_err(|err| format!("cannot query address book: {}", err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("cannot read address book row: {}", err))?;

    serde_json::to_string(&addresses)
        .map_err(|err| format!("cannot serialize addresses as JSON: {}", err))
}

#[cfg(test)]
mod address_book_tests {
    use super::get_addresses_from_addressbook;
    use rusqlite::Connection;
    use serde_json::json;

    #[test]
    fn returns_addresses_as_json() {
        let temp_dir = tempfile::tempdir().expect("create temporary directory");
        let address_book = temp_dir.path().join("AddressBook-v22.abcddb");
        let connection =
            Connection::open(&address_book).expect("create temporary address book database");

        connection
            .execute_batch(
                "
                CREATE TABLE ZABCDRECORD (
                    Z_PK INTEGER PRIMARY KEY,
                    ZUNIQUEID TEXT,
                    ZTYPE INTEGER,
                    ZFIRSTNAME TEXT,
                    ZMIDDLENAME TEXT,
                    ZLASTNAME TEXT,
                    ZNICKNAME TEXT,
                    ZORGANIZATION TEXT,
                    ZJOBTITLE TEXT
                );
                CREATE TABLE ZABCDPHONENUMBER (
                    Z_PK INTEGER PRIMARY KEY,
                    ZOWNER INTEGER,
                    ZFULLNUMBER TEXT
                );
                CREATE TABLE ZABCDEMAILADDRESS (
                    Z_PK INTEGER PRIMARY KEY,
                    ZOWNER INTEGER,
                    ZADDRESS TEXT
                );
                INSERT INTO ZABCDRECORD (
                    Z_PK, ZUNIQUEID, ZTYPE, ZFIRSTNAME, ZMIDDLENAME, ZLASTNAME,
                    ZNICKNAME, ZORGANIZATION, ZJOBTITLE
                ) VALUES
                    (1, 'record-ada', 0, 'Ada', 'Byron', 'Lovelace', 'Ada',
                     'Analytical Engines', 'Mathematician'),
                    (2, 'record-grace', 0, 'Grace', NULL, 'Hopper', 'Amazing Grace',
                     'Navy', 'Rear Admiral'),
                    (3, 'record-alan', 0, 'Alan', 'Mathison', 'Turing', NULL,
                     'Bletchley Park', 'Cryptanalyst');
                INSERT INTO ZABCDPHONENUMBER (Z_PK, ZOWNER, ZFULLNUMBER) VALUES
                    (1, 1, '+1-555-0101'),
                    (2, 2, '+1-555-0102'),
                    (3, 3, '+1-555-0103');
                INSERT INTO ZABCDEMAILADDRESS (Z_PK, ZOWNER, ZADDRESS) VALUES
                    (1, 1, 'ada@example.test'),
                    (2, 2, 'grace@example.test'),
                    (3, 3, 'alan@example.test');
                ",
            )
            .expect("populate temporary address book database");
        drop(connection);

        let actual = get_addresses_from_addressbook(&address_book)
            .expect("get addresses from temporary address book");
        let actual_json: serde_json::Value =
            serde_json::from_str(&actual).expect("returned string should be valid JSON");
        let expected_json = json!([
            {
                "contact_id": 2,
                "contact_unique_id": "record-grace",
                "record_type": 0,
                "first_name": "Grace",
                "middle_name": null,
                "last_name": "Hopper",
                "nickname": "Amazing Grace",
                "organization": "Navy",
                "job_title": "Rear Admiral",
                "phone_numbers": "+1-555-0102",
                "email_addresses": "grace@example.test"
            },
            {
                "contact_id": 1,
                "contact_unique_id": "record-ada",
                "record_type": 0,
                "first_name": "Ada",
                "middle_name": "Byron",
                "last_name": "Lovelace",
                "nickname": "Ada",
                "organization": "Analytical Engines",
                "job_title": "Mathematician",
                "phone_numbers": "+1-555-0101",
                "email_addresses": "ada@example.test"
            },
            {
                "contact_id": 3,
                "contact_unique_id": "record-alan",
                "record_type": 0,
                "first_name": "Alan",
                "middle_name": "Mathison",
                "last_name": "Turing",
                "nickname": null,
                "organization": "Bletchley Park",
                "job_title": "Cryptanalyst",
                "phone_numbers": "+1-555-0103",
                "email_addresses": "alan@example.test"
            }
        ]);

        assert_eq!(actual_json, expected_json);
    }
}

/// Gets all addresses from AddressBook-v22.abcddb files listed in a SHA-256 manifest.
/// # Arguments
/// * `manifest_path` - The path to the SHA-256 manifest file. Manifest lists relative paths.
/// * `working_dir` - The path to the working copy of the collected evidence.
/// # Returns
/// * `Ok(String)` - Addresses in JSON format if successful.
/// * `Err(String)` - An error message if the operation fails.
pub fn get_all_addresses(manifest_path: &Path, working_dir: &Path) -> Result<String, String> {
    let manifest_contents = fs::read_to_string(manifest_path).map_err(|err| {
        format!(
            "cannot read hash manifest '{}': {}",
            manifest_path.display(),
            err
        )
    })?;
    let mut all_addresses = Vec::new();

    for (line_number, line) in manifest_contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }

        let (_, relative_path) = parse_hash_manifest_line(line, line_number + 1, manifest_path)?;
        if relative_path
            .file_name()
            .is_none_or(|file_name| file_name != "AddressBook-v22.abcddb")
        {
            continue;
        }

        let address_book_path = working_dir.join(&relative_path);
        let addresses_json = get_addresses_from_addressbook(&address_book_path).map_err(|err| {
            format!(
                "cannot get addresses from address book '{}': {}",
                address_book_path.display(),
                err
            )
        })?;
        let mut addresses: Vec<serde_json::Value> =
            serde_json::from_str(&addresses_json).map_err(|err| {
                format!(
                    "cannot parse addresses from address book '{}' as JSON: {}",
                    address_book_path.display(),
                    err
                )
            })?;
        all_addresses.append(&mut addresses);
    }

    serde_json::to_string(&all_addresses)
        .map_err(|err| format!("cannot serialize all addresses as JSON: {}", err))
}

/// Gets all messages from chat.db
/// # Arguments
/// * `chat_db` - The path to the iMessage chat database file.
/// # Returns
/// * `Ok(String)` - Messages in JSON format if successful.
/// * `Err(String)` - An error message if the operation fails.
pub fn get_all_messages(chat_db: &Path) -> Result<String, String> {
    #[derive(serde::Serialize)]
    struct Message {
        timestamp: Option<String>,
        chat: Option<String>,
        sender: Option<String>,
        message: Option<String>,
        attachment: Option<String>,
    }

    let connection = Connection::open(chat_db)
        .map_err(|err| format!("cannot open chat database '{}': {}", chat_db.display(), err))?;
    let mut statement = connection
        .prepare(include_str!("../query/msg-query.sql"))
        .map_err(|err| format!("cannot prepare message query: {}", err))?;
    let messages = statement
        .query_map([], |row| {
            let timestamp: Option<String> = row.get(0)?;
            Ok(Message {
                timestamp,
                chat: row.get(1)?,
                sender: row.get(2)?,
                message: row.get(3)?,
                attachment: row.get(4)?,
            })
        })
        .map_err(|err| format!("cannot query messages: {}", err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("cannot read message row: {}", err))?;

    serde_json::to_string(&messages)
        .map_err(|err| format!("cannot serialize messages as JSON: {}", err))
}

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Contact {
    pub contact_id: Option<serde_json::Value>,
    pub contact_unique_id: Option<String>,
    pub email_addresses: Option<String>,
    pub first_name: Option<String>,
    pub job_title: Option<String>,
    pub last_name: Option<String>,
    pub middle_name: Option<String>,
    pub nickname: Option<String>,
    pub organization: Option<String>,
    pub phone_numbers: Option<String>,
    pub record_type: Option<serde_json::Value>,
}

type EmailOrPhone = String;
pub type ContactLookup = HashMap<EmailOrPhone, Contact>;

/// Parses the addresses JSON to create a lookup of emails and phone numbers to contacts
/// # Arguments
/// * `json_str` - The JSON string containing the addresses data.
/// # Returns
/// * `Ok(ContactLookup)` - A map where keys are emails/phone numbers and the values are the Contact struct.
pub fn unmarshal_addresses_from_json(json_str: &str) -> Result<ContactLookup, String> {
    let contacts: Vec<serde_json::Value> = serde_json::from_str(json_str)
        .map_err(|err| format!("cannot parse addresses JSON: {}", err))?;
    let mut addresses = HashMap::new();

    for (contact_index, contact) in contacts.iter().enumerate() {
        let contact_object = contact
            .as_object()
            .ok_or_else(|| format!("address at index {} is not a JSON object", contact_index))?;
        
        let contact_struct: Contact = serde_json::from_value(contact.clone()).map_err(|err| {
            format!("cannot deserialize address at index {}: {}", contact_index, err)
        })?;

        for field in ["email_addresses", "phone_numbers"] {
            let Some(value) = contact_object.get(field) else {
                continue;
            };
            let Some(values) = value.as_str() else {
                if value.is_null() {
                    continue;
                }
                return Err(format!(
                    "address at index {} has a non-string {} field",
                    contact_index, field
                ));
            };

            for value in values
                .split('|')
                .map(str::trim)
                .filter(|value| !value.is_empty())
            {
                let key = if field == "email_addresses" {
                    value.to_lowercase()
                } else {
                    let digits = value
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>();
                    match digits.chars().nth(0) {
                        Some('1') => format!("+{}", digits),
                        Some('+') => digits,
                        Some(_) => format!("+1{}", digits),
                        None => String::new(),
                    }
                };
                if !key.is_empty() {
                    addresses.insert(key, contact_struct.clone());
                }
            }
        }
    }

    Ok(addresses)
}

#[cfg(test)]
mod unmarshal_addresses_tests {
    use super::{unmarshal_addresses_from_json, Contact};

    #[test]
    fn maps_fixture_emails_and_phone_numbers_to_contacts() {
        let addresses_json = include_str!("../tests/fixtures/addresses.json");
        let phone_email_json = include_str!("../tests/fixtures/phone_email.json");
        let addresses: Vec<serde_json::Value> =
            serde_json::from_str(addresses_json).expect("parse addresses fixture");
        let phone_email: serde_json::Value =
            serde_json::from_str(phone_email_json).expect("parse phone and email fixture");
        
        let actual =
            unmarshal_addresses_from_json(addresses_json).expect("parse addresses fixture data");
        assert_eq!(actual.len(), 19);

        for (email_index, contact_index) in [(0, 3), (2, 4), (4, 5)] {
            let email = phone_email["emails"][email_index]
                .as_str()
                .expect("email fixture entry should be a string");
            let expected_contact: Contact = serde_json::from_value(addresses[contact_index].clone())
                .expect("serialize contact");
            assert_eq!(actual.get(email), Some(&expected_contact));
        }

        for (phone_index, contact_index) in [(0, 0), (2, 1), (4, 2)] {
            let phone = phone_email["phone"][phone_index]
                .as_str()
                .expect("phone fixture entry should be a string");
            let digits = phone
                .chars()
                .filter(|character| character.is_ascii_digit())
                .collect::<String>();
            let normalized_phone = if digits.starts_with('1') {
                format!("+{}", digits)
            } else {
                format!("+1{}", digits)
            };
            let expected_contact: Contact = serde_json::from_value(addresses[contact_index].clone())
                .expect("serialize contact");
            assert_eq!(actual.get(&normalized_phone), Some(&expected_contact));
        }
    }

}

#[derive(Clone)]
pub struct Message {
    timestamp: Option<DateTime<Utc>>,
    chat: Option<String>,
    sender: Option<String>,
    message: Option<String>,
    attachment: Option<String>,
}

/// Unmarshals messages from JSON string to a vector of Message structs
/// # Arguments
/// * `json_str` - The JSON string containing the messages data.
/// # Returns
/// * `Ok(Vec<Message>)` - A vector of Message structs if successful.
/// * `Err(String)` - An error message if the unmarshalling fails.
pub fn unmarshal_messages_from_json(json_str: &str) -> Result<Vec<Message>, String> {
    #[derive(serde::Deserialize)]
    struct SerializedMessage {
        timestamp: Option<String>,
        chat: Option<String>,
        sender: Option<String>,
        message: Option<String>,
        attachment: Option<String>,
    }

    let serialized_messages: Vec<SerializedMessage> = serde_json::from_str(json_str)
        .map_err(|err| format!("cannot parse messages JSON: {}", err))?;

    serialized_messages
        .into_iter()
        .enumerate()
        .map(|(index, message)| {
            let timestamp = message
                .timestamp
                .as_deref()
                .map(|timestamp| {
                    DateTime::parse_from_rfc3339(timestamp)
                        .map(|timestamp| timestamp.with_timezone(&Utc))
                        .map_err(|err| {
                            format!(
                                "cannot parse timestamp for message at index {}: {}",
                                index, err
                            )
                        })
                })
                .transpose()?;

            Ok(Message {
                timestamp,
                chat: message.chat,
                sender: message.sender,
                message: message.message,
                attachment: message.attachment,
            })
        })
        .collect()
}

#[cfg(test)]
mod unmarshal_messages_tests {
    use super::unmarshal_messages_from_json;
    use chrono::SecondsFormat;

    #[test]
    fn unmarshals_fixture_messages() {
        let messages_json = include_str!("../tests/fixtures/messages.json");
        let expected: Vec<serde_json::Value> =
            serde_json::from_str(messages_json).expect("parse messages fixture");
        let actual =
            unmarshal_messages_from_json(messages_json).expect("unmarshal messages fixture");

        assert_eq!(expected.len(), 20);
        assert_eq!(actual.len(), expected.len());

        for (actual, expected) in actual.iter().zip(&expected) {
            assert_eq!(
                actual
                    .timestamp
                    .as_ref()
                    .expect("fixture timestamp should be present")
                    .to_rfc3339_opts(SecondsFormat::Secs, true),
                expected["timestamp"]
                    .as_str()
                    .expect("fixture timestamp should be a string")
            );
            assert_eq!(
                actual.chat.as_deref(),
                Some(
                    expected["chat"]
                        .as_str()
                        .expect("fixture chat should be a string")
                )
            );
            assert_eq!(actual.sender.as_deref(), expected["sender"].as_str());
            assert_eq!(
                actual.message.as_deref(),
                expected["message"]
                    .as_str()
                    .expect("fixture message should be a string")
                    .into()
            );
            assert_eq!(
                actual.attachment.as_deref(),
                expected["attachment"].as_str()
            );
        }
    }

    #[test]
    fn unmarshals_null_sender() {
        let messages_json = r#"[{
            "timestamp": "2024-10-02T11:28:51Z",
            "chat": null,
            "sender": null,
            "message": "Message with no sender",
            "attachment": null
        }]"#;

        let actual =
            unmarshal_messages_from_json(messages_json).expect("unmarshal null sender message");

        assert_eq!(actual.len(), 1);
        assert_eq!(actual[0].sender, None);
        assert_eq!(actual[0].message.as_deref(), Some("Message with no sender"));
    }

    #[test]
    fn unmarshals_all_null_or_missing_fields() {
        let actual = unmarshal_messages_from_json(
            r#"[{"timestamp":null,"chat":null,"sender":null,"message":null,"attachment":null},{}]"#,
        )
        .expect("unmarshal messages with null and missing fields");

        assert_eq!(actual.len(), 2);
        for message in actual {
            assert!(message.timestamp.is_none());
            assert!(message.chat.is_none());
            assert!(message.sender.is_none());
            assert!(message.message.is_none());
            assert!(message.attachment.is_none());
        }
    }
}

pub type MsgContactTuple = (Message, Option<Contact>);
/// Joins messages with their corresponding contacts based on the sender's email or phone number.
/// # Arguments
/// * `messages` - A slice of Message structs to be joined with contacts.
/// * `addresses` - A ContactLookup where keys are emails/phone numbers and values are Contact structs.
/// # Returns
/// * `Vec<(Message, Option<Contact>)>` - A vector of tuples where each tuple contains a Message and an optional Contact struct. If a contact is found for the message's sender, the second element of the tuple will be `Some(contact)`, otherwise it will be `None`.
pub fn join_messages_to_contacts(
    messages: &[Message],
    addresses: &ContactLookup,
) -> Vec<MsgContactTuple> {
    messages
        .iter()
        .cloned()
        .map(|message| {
            let sender_key = message.sender.as_deref().and_then(|sender| {
                if sender.contains('@') {
                    Some(sender.to_lowercase())
                } else {
                    let digits = sender
                        .chars()
                        .filter(char::is_ascii_digit)
                        .collect::<String>();
                    if digits.is_empty() {
                        None
                    } else if digits.starts_with('1') {
                        Some(format!("+{}", digits))
                    } else {
                        Some(format!("+1{}", digits))
                    }
                }
            });
            let contact = sender_key.and_then(|key| addresses.get(&key).cloned());
            (message, contact)
        })
        .collect()
}

#[cfg(test)]
mod join_messages_tests {
    use super::{Message, join_messages_to_contacts, Contact};
    use chrono::{DateTime, Utc};
    use std::collections::HashMap;

    #[test]
    fn joins_messages_using_normalized_email_and_phone_senders() {
        let timestamp = Some(
            DateTime::parse_from_rfc3339("2024-10-02T11:28:51Z")
                .expect("valid test timestamp")
                .with_timezone(&Utc),
        );
        let messages = vec![
            Message {
                timestamp,
                chat: None,
                sender: Some("ALICE@EXAMPLE.COM".to_string()),
                message: Some("Email message".to_string()),
                attachment: None,
            },
            Message {
                timestamp,
                chat: None,
                sender: Some("1 (555) 123-4567".to_string()),
                message: Some("Phone message".to_string()),
                attachment: None,
            },
            Message {
                timestamp,
                chat: None,
                sender: Some("unknown@example.com".to_string()),
                message: Some("Unmatched message".to_string()),
                attachment: None,
            },
        ];
        let addresses = HashMap::from([
            ("alice@example.com".to_string(), Contact {
                first_name: Some("Alice".to_string()),
                contact_id: None,
                contact_unique_id: None,
                email_addresses: None,
                job_title: None,
                last_name: None,
                middle_name: None,
                nickname: None,
                organization: None,
                phone_numbers: None,
                record_type: None,
            }),
            ("+15551234567".to_string(), Contact {
                first_name: Some("Phone Contact".to_string()),
                contact_id: None,
                contact_unique_id: None,
                email_addresses: None,
                job_title: None,
                last_name: None,
                middle_name: None,
                nickname: None,
                organization: None,
                phone_numbers: None,
                record_type: None,
            }),
        ]);

        let joined = join_messages_to_contacts(&messages, &addresses);

        assert_eq!(joined.len(), 3);
        assert_eq!(joined[0].1.as_ref().and_then(|c| c.first_name.as_deref()), Some("Alice"));
        assert_eq!(joined[1].1.as_ref().and_then(|c| c.first_name.as_deref()), Some("Phone Contact"));
        assert_eq!(joined[2].1, None);
        assert_eq!(joined[2].0.message.as_deref(), Some("Unmatched message"));
    }
}

/// Marshals a vector of (Message, Option<String>) tuples into a JSON array.
/// Each tuple is represented as a JSON object with the message fields and an optional contact field.
/// # Arguments
/// * `joined` - A vector of tuples where each tuple contains a Message and an optional contact JSON string.
/// # Returns
/// * `serde_json::Value` - A JSON array where each element is a JSON object representing a message and its associated contact (if any).
pub fn marshal_msg_contact_tuple_to_json(joined: &[MsgContactTuple]) -> serde_json::Value {
    serde_json::Value::Array(
        joined
            .iter()
            .map(|(message, contact)| {
                serde_json::json!({
                    "timestamp": message.timestamp.as_ref().map(|timestamp| {
                        timestamp.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
                    }),
                    "chat": message.chat,
                    "sender": message.sender,
                    "message": message.message,
                    "attachment": message.attachment,
                    "contact": contact,
                })
            })
            .collect(),
    )
}

#[cfg(test)]
mod marshal_msg_contact_tuple_tests {
    use super::{Message, MsgContactTuple, marshal_msg_contact_tuple_to_json, Contact};
    use chrono::{DateTime, Utc};
    use serde_json::json;

    #[test]
    fn marshals_messages_with_optional_contacts_to_json() {
        let timestamp = Some(
            DateTime::parse_from_rfc3339("2024-10-02T11:28:51Z")
                .expect("valid test timestamp")
                .with_timezone(&Utc),
        );
        let joined: Vec<MsgContactTuple> = vec![
            (
                Message {
                    timestamp,
                    chat: Some("chat333086607288203406".to_string()),
                    sender: Some("+15551234567".to_string()),
                    message: Some("Hello".to_string()),
                    attachment: None,
                },
                Some(Contact {
                    first_name: Some("Alice".to_string()),
                    contact_id: None,
                    contact_unique_id: None,
                    email_addresses: None,
                    job_title: None,
                    last_name: None,
                    middle_name: None,
                    nickname: None,
                    organization: None,
                    phone_numbers: None,
                    record_type: None,
                }),
            ),
            (
                Message {
                    timestamp,
                    chat: None,
                    sender: Some("unknown@example.com".to_string()),
                    message: Some("No matching contact".to_string()),
                    attachment: Some("image.png".to_string()),
                },
                None,
            ),
        ];

        let result = marshal_msg_contact_tuple_to_json(&joined);
        let expected = json!([
            {
                "timestamp": "2024-10-02T11:28:51Z",
                "chat": "chat333086607288203406",
                "sender": "+15551234567",
                "message": "Hello",
                "attachment": null,
                "contact": {
                    "first_name": "Alice",
                    "contact_id": null,
                    "contact_unique_id": null,
                    "email_addresses": null,
                    "job_title": null,
                    "last_name": null,
                    "middle_name": null,
                    "nickname": null,
                    "organization": null,
                    "phone_numbers": null,
                    "record_type": null
                }
            },
            {
                "timestamp": "2024-10-02T11:28:51Z",
                "chat": null,
                "sender": "unknown@example.com",
                "message": "No matching contact",
                "attachment": "image.png",
                "contact": null
            }
        ]);
        assert_eq!(result, expected);
    }
}
