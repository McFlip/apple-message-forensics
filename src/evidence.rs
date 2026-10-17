use crate::logger;
use hex;
use rusqlite::{Connection, OpenFlags, OptionalExtension, Transaction, TransactionBehavior};
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
use uuid::Uuid;

const MAX_COPY_ATTEMPTS: usize = 3;
const HASH_RETRY_DELAY: Duration = Duration::from_secs(60);
const REQUIRED_TABLES: [&str; 3] = ["message", "chat", "attachment"];

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
        output.join("delivery"),
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

pub fn write_evidence_hashes(home_dir: &Path, output_dir: &Path) -> Result<PathBuf, String> {
    let out_file_path = output_dir
        .join("evidence")
        .join("working_copy")
        .join("hash_manifest.txt");
    let mut manifest_entries: Vec<(String, String)> = Vec::new();

    let evidence_files = find_evidence_files(home_dir)?;

    for relative_path in evidence_files.iter() {
        let absolute_path = home_dir.join(relative_path);

        match calculate_sha256(&absolute_path) {
            Ok(hash) => {
                manifest_entries.push((hash, relative_path.display().to_string()));
            }
            Err(e) => {
                logger::warn_fmt(format_args!(
                    "Could not hash file {}: {}",
                    relative_path.display(),
                    e
                ));
            }
        }
    }

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

pub fn write_report_hashes(report_dir: &Path) -> Result<PathBuf, String> {
    let out_file_path = report_dir.join("hash_manifest.txt");
    let mut manifest_entries: Vec<(String, String)> = Vec::new();

    fn visit_report_directory(
        dir: &Path,
        base: &Path,
        entries: &mut Vec<(String, String)>,
    ) -> Result<(), String> {
        let read_dir = fs::read_dir(dir)
            .map_err(|err| format!("cannot read report directory '{}': {}", dir.display(), err))?;
        for entry in read_dir {
            let entry = entry.map_err(|err| {
                format!(
                    "cannot read an entry in report directory '{}': {}",
                    dir.display(),
                    err
                )
            })?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|err| {
                format!("cannot inspect report path '{}': {}", path.display(), err)
            })?;
            if file_type.is_dir() {
                visit_report_directory(&path, base, entries)?;
            } else if file_type.is_file() {
                let relative = path.strip_prefix(base).map_err(|err| {
                    format!(
                        "report path '{}' is not beneath report directory '{}': {}",
                        path.display(),
                        base.display(),
                        err
                    )
                })?;
                let hash = calculate_sha256(&path)?;
                entries.push((hash, relative.display().to_string()));
            }
        }
        Ok(())
    }

    visit_report_directory(report_dir, report_dir, &mut manifest_entries)?;

    // Sort for deterministic output
    manifest_entries.sort_by(|a, b| a.1.cmp(&b.1));

    let mut out_file = fs::File::create(&out_file_path).map_err(|err| {
        format!(
            "cannot create hash manifest file '{}': {}",
            out_file_path.display(),
            err
        )
    })?;

    for (hash, rel_path) in manifest_entries {
        writeln!(out_file, "{}  {}", hash, rel_path).map_err(|err| {
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

#[derive(serde::Serialize)]
struct AddressBookExport {
    format: &'static str,
    source: AddressBookMetadata,
    contacts: Vec<ExportedContact>,
    target_losses: [&'static str; 3],
}

#[derive(serde::Serialize)]
struct AddressBookMetadata {
    database_sha256: String,
    platform: &'static str,
    tables: [&'static str; 3],
}

#[derive(serde::Serialize)]
struct ExportedContact {
    source: ExportedRecordSource,
    name: ExportedName,
    organization: Option<String>,
    phones: Vec<ExportedPhone>,
    emails: Vec<ExportedEmail>,
}

#[derive(serde::Serialize)]
struct ExportedRecordSource {
    record_pk: i64,
    record_uid: Option<String>,
}

#[derive(serde::Serialize)]
struct ExportedName {
    first: Option<String>,
    middle: Option<String>,
    last: Option<String>,
}

#[derive(serde::Serialize)]
struct ExportedValueSource {
    row_pk: i64,
    row_uid: Option<String>,
    is_primary: Option<bool>,
}

#[derive(serde::Serialize)]
struct ExportedPhone {
    source: ExportedValueSource,
    value: Option<String>,
    label: Option<String>,
    #[serde(rename = "order")]
    ordering_index: Option<i64>,
    last_four: Option<String>,
}

#[derive(serde::Serialize)]
struct ExportedEmail {
    source: ExportedValueSource,
    value: Option<String>,
    normalized_value: Option<String>,
    label: Option<String>,
    #[serde(rename = "order")]
    ordering_index: Option<i64>,
}

pub fn create_cellebrite_output(output_dir: &Path) -> Result<(), String> {
    if !output_dir.is_dir() {
        return Err(format!(
            "output directory does not exist: {}",
            output_dir.display()
        ));
    }

    let working_copy = output_dir.join("evidence").join("working_copy");
    let manifest_path = working_copy.join("hash_manifest.txt");
    let manifest_contents = fs::read_to_string(&manifest_path).map_err(|err| {
        format!(
            "cannot read hash manifest '{}': {}",
            manifest_path.display(),
            err
        )
    })?;

    let mut manifest_hashes = HashMap::new();
    for (line_number, line) in manifest_contents.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let (hash, relative_path) =
            parse_hash_manifest_line(line, line_number + 1, &manifest_path)?;
        if manifest_hashes
            .insert(relative_path.clone(), hash)
            .is_some()
        {
            return Err(format!(
                "duplicate path '{}' in hash manifest '{}'",
                relative_path.display(),
                manifest_path.display()
            ));
        }
    }

    let cellebrite_dir = output_dir.join("cellebrite");
    let library_dir = cellebrite_dir
        .join("private")
        .join("var")
        .join("mobile")
        .join("Library");

    copy_verified_tree(
        &working_copy
            .join("Library")
            .join("Application Support")
            .join("AddressBook"),
        Path::new("Library/Application Support/AddressBook"),
        &library_dir.join("AddressBook"),
        &manifest_hashes,
    )?;
    let sms_dir = library_dir.join("SMS");
    copy_verified_tree(
        &working_copy.join("Library").join("Messages"),
        Path::new("Library/Messages"),
        &sms_dir,
        &manifest_hashes,
    )?;

    let lib_addr_dir = &library_dir.join("AddressBook");
    let source_path = &lib_addr_dir.join("AddressBook-v22.abcddb");
    let new_name = &lib_addr_dir.join("AddressBook.sqlitedb");
    let json_dir = output_dir.join("json");
    fs::create_dir_all(&json_dir).map_err(|err| {
        format!(
            "cannot create JSON output directory '{}': {}",
            json_dir.display(),
            err
        )
    })?;
    let contacts_json = json_dir.join("cellebrite_addresses.json");
    let exported = export_address_book(source_path)?;
    write_address_book_json(&exported, &contacts_json)?;
    let schema_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("schema/iPhoneSyntheticSchema.sql");
    create_synthetic_address_book(new_name, &schema_path, &exported)?;
    fs::remove_file(source_path).map_err(|err| {
        format!(
            "cannot remove converted source AddressBook database '{}': {}",
            source_path.display(),
            err
        )
    })?;
    set_read_only(new_name).map_err(|err| {
        format!(
            "cannot set read-only permissions on generated AddressBook database '{}': {}",
            new_name.display(),
            err
        )
    })?;

    for (source_name, destination_name) in [
        ("chat.db", "sms.db"),
        ("chat.db-wal", "sms.db-wal"),
        ("chat.db-shm", "sms.db-shm"),
    ] {
        let source_path = sms_dir.join(source_name);
        if source_path.exists() {
            fs::rename(&source_path, sms_dir.join(destination_name)).map_err(|err| {
                format!(
                    "cannot rename '{}' to '{}': {}",
                    source_path.display(),
                    sms_dir.join(destination_name).display(),
                    err
                )
            })?;
        } else if source_name == "chat.db" {
            return Err(format!(
                "Messages database does not exist: {}",
                source_path.display()
            ));
        }
    }

    Ok(())
}

fn export_address_book(source_path: &Path) -> Result<AddressBookExport, String> {
    let source_hash = calculate_sha256(source_path)?;
    let connection = Connection::open_with_flags(source_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|err| {
            format!(
                "cannot open source AddressBook database '{}': {}",
                source_path.display(),
                err
            )
        })?;
    connection
        .pragma_update(None, "query_only", true)
        .map_err(|err| format!("cannot enable read-only AddressBook queries: {err}"))?;

    for (table, required_columns) in [
        (
            "ZABCDRECORD",
            &[
                "Z_PK",
                "ZUNIQUEID",
                "ZFIRSTNAME",
                "ZMIDDLENAME",
                "ZLASTNAME",
                "ZORGANIZATION",
            ][..],
        ),
        (
            "ZABCDPHONENUMBER",
            &[
                "Z_PK",
                "ZUNIQUEID",
                "ZOWNER",
                "ZFULLNUMBER",
                "ZLABEL",
                "ZORDERINGINDEX",
                "ZISPRIMARY",
                "ZLASTFOURDIGITS",
            ][..],
        ),
        (
            "ZABCDEMAILADDRESS",
            &[
                "Z_PK",
                "ZUNIQUEID",
                "ZOWNER",
                "ZADDRESS",
                "ZADDRESSNORMALIZED",
                "ZLABEL",
                "ZORDERINGINDEX",
                "ZISPRIMARY",
            ][..],
        ),
    ] {
        require_address_book_columns(&connection, table, required_columns)?;
    }

    let records = {
        let mut statement = connection
            .prepare(
                "SELECT
                    r.Z_PK,
                    r.ZUNIQUEID,
                    r.ZFIRSTNAME,
                    r.ZMIDDLENAME,
                    r.ZLASTNAME,
                    r.ZORGANIZATION
                 FROM ZABCDRECORD AS r
                 WHERE r.ZFIRSTNAME IS NOT NULL
                    OR r.ZMIDDLENAME IS NOT NULL
                    OR r.ZLASTNAME IS NOT NULL
                    OR r.ZORGANIZATION IS NOT NULL
                    OR EXISTS (
                        SELECT 1 FROM ZABCDPHONENUMBER AS p WHERE p.ZOWNER = r.Z_PK
                    )
                    OR EXISTS (
                        SELECT 1 FROM ZABCDEMAILADDRESS AS e WHERE e.ZOWNER = r.Z_PK
                    )
                 ORDER BY r.Z_PK",
            )
            .map_err(|err| format!("cannot prepare AddressBook record query: {err}"))?;
        statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, Option<String>>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, Option<String>>(3)?,
                    row.get::<_, Option<String>>(4)?,
                    row.get::<_, Option<String>>(5)?,
                ))
            })
            .map_err(|err| format!("cannot query AddressBook records: {err}"))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| format!("cannot read an AddressBook record: {err}"))?
    };

    let mut contacts = Vec::with_capacity(records.len());
    for (record_pk, record_uid, first, middle, last, organization) in records {
        contacts.push(ExportedContact {
            source: ExportedRecordSource {
                record_pk,
                record_uid,
            },
            name: ExportedName {
                first,
                middle,
                last,
            },
            organization,
            phones: export_phone_values(&connection, record_pk)?,
            emails: export_email_values(&connection, record_pk)?,
        });
    }

    Ok(AddressBookExport {
        format: "apple-addressbook-forensic-v1",
        source: AddressBookMetadata {
            database_sha256: source_hash,
            platform: "macOS",
            tables: ["ZABCDRECORD", "ZABCDPHONENUMBER", "ZABCDEMAILADDRESS"],
        },
        contacts,
        target_losses: [
            "Mac primary-value flags are preserved in JSON but have no direct ABMultiValue equivalent.",
            "Mac record and child unique IDs are preserved as source provenance only.",
            "Images, postal addresses, sync metadata, account metadata, history, and search artifacts are omitted.",
        ],
    })
}

fn require_address_book_columns(
    connection: &Connection,
    table: &str,
    required_columns: &[&str],
) -> Result<(), String> {
    let mut statement = connection
        .prepare(&format!("PRAGMA table_info({table})"))
        .map_err(|err| format!("cannot inspect source table {table}: {err}"))?;
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(|err| format!("cannot query source table {table} columns: {err}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("cannot read source table {table} columns: {err}"))?;
    let missing: Vec<_> = required_columns
        .iter()
        .filter(|column| !columns.iter().any(|actual| actual == **column))
        .copied()
        .collect();

    if missing.is_empty() {
        Ok(())
    } else {
        Err(format!(
            "{table} is missing required columns: {}",
            missing.join(", ")
        ))
    }
}

fn export_phone_values(
    connection: &Connection,
    record_pk: i64,
) -> Result<Vec<ExportedPhone>, String> {
    let mut statement = connection
        .prepare(
            "SELECT
                Z_PK, ZUNIQUEID, ZFULLNUMBER, ZLABEL, ZORDERINGINDEX, ZISPRIMARY, ZLASTFOURDIGITS
             FROM ZABCDPHONENUMBER
             WHERE ZOWNER = ?1
             ORDER BY ZORDERINGINDEX, Z_PK",
        )
        .map_err(|err| format!("cannot prepare phone query for contact {record_pk}: {err}"))?;
    statement
        .query_map([record_pk], |row| {
            Ok(ExportedPhone {
                source: ExportedValueSource {
                    row_pk: row.get(0)?,
                    row_uid: row.get(1)?,
                    is_primary: row
                        .get::<_, Option<i64>>(5)?
                        .map(|is_primary| is_primary != 0),
                },
                value: row.get(2)?,
                label: row.get(3)?,
                ordering_index: row.get(4)?,
                last_four: row.get(6)?,
            })
        })
        .map_err(|err| format!("cannot query phone values for contact {record_pk}: {err}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("cannot read phone value for contact {record_pk}: {err}"))
}

fn export_email_values(
    connection: &Connection,
    record_pk: i64,
) -> Result<Vec<ExportedEmail>, String> {
    let mut statement = connection
        .prepare(
            "SELECT
                Z_PK, ZUNIQUEID, ZADDRESS, ZADDRESSNORMALIZED, ZLABEL, ZORDERINGINDEX, ZISPRIMARY
             FROM ZABCDEMAILADDRESS
             WHERE ZOWNER = ?1
             ORDER BY ZORDERINGINDEX, Z_PK",
        )
        .map_err(|err| format!("cannot prepare email query for contact {record_pk}: {err}"))?;
    statement
        .query_map([record_pk], |row| {
            Ok(ExportedEmail {
                source: ExportedValueSource {
                    row_pk: row.get(0)?,
                    row_uid: row.get(1)?,
                    is_primary: row
                        .get::<_, Option<i64>>(6)?
                        .map(|is_primary| is_primary != 0),
                },
                value: row.get(2)?,
                normalized_value: row.get(3)?,
                label: row.get(4)?,
                ordering_index: row.get(5)?,
            })
        })
        .map_err(|err| format!("cannot query email values for contact {record_pk}: {err}"))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("cannot read email value for contact {record_pk}: {err}"))
}

fn write_address_book_json(export: &AddressBookExport, output_path: &Path) -> Result<(), String> {
    let temporary_path = output_path.with_extension("json.tmp");
    let mut file = fs::File::create(&temporary_path).map_err(|err| {
        format!(
            "cannot create intermediate AddressBook JSON '{}': {}",
            temporary_path.display(),
            err
        )
    })?;
    serde_json::to_writer_pretty(&mut file, export)
        .map_err(|err| format!("cannot serialize intermediate AddressBook JSON: {err}"))?;
    file.write_all(b"\n").map_err(|err| {
        format!(
            "cannot finish intermediate AddressBook JSON '{}': {}",
            temporary_path.display(),
            err
        )
    })?;
    file.sync_all().map_err(|err| {
        format!(
            "cannot sync intermediate AddressBook JSON '{}': {}",
            temporary_path.display(),
            err
        )
    })?;
    fs::rename(&temporary_path, output_path).map_err(|err| {
        format!(
            "cannot write intermediate AddressBook JSON '{}': {}",
            output_path.display(),
            err
        )
    })
}

fn create_synthetic_address_book(
    database_path: &Path,
    schema_path: &Path,
    export: &AddressBookExport,
) -> Result<(), String> {
    if database_path.exists() {
        return Err(format!(
            "refusing to overwrite existing generated AddressBook database: {}",
            database_path.display()
        ));
    }
    let schema = fs::read_to_string(schema_path).map_err(|err| {
        format!(
            "cannot read synthetic AddressBook schema '{}': {}",
            schema_path.display(),
            err
        )
    })?;
    let mut connection = Connection::open(database_path).map_err(|err| {
        format!(
            "cannot create synthetic AddressBook database '{}': {}",
            database_path.display(),
            err
        )
    })?;
    connection
        .execute_batch(&schema)
        .map_err(|err| format!("cannot initialize synthetic AddressBook schema: {err}"))?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|err| format!("cannot check synthetic AddressBook database integrity: {err}"))?;
    if integrity != "ok" {
        return Err(format!(
            "synthetic AddressBook database integrity check failed: {integrity}"
        ));
    }

    initialize_synthetic_store(&connection, &export.source.database_sha256)?;
    require_synthetic_tables(&connection)?;
    let person_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM ABPerson", [], |row| row.get(0))
        .map_err(|err| format!("cannot inspect synthetic AddressBook people: {err}"))?;
    let multivalue_count: i64 = connection
        .query_row("SELECT COUNT(*) FROM ABMultiValue", [], |row| row.get(0))
        .map_err(|err| format!("cannot inspect synthetic AddressBook values: {err}"))?;
    if person_count != 0 || multivalue_count != 0 {
        return Err("synthetic AddressBook destination must be empty".to_owned());
    }
    let store_exists: bool = connection
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM ABStore WHERE ROWID = 0)",
            [],
            |row| row.get(0),
        )
        .map_err(|err| format!("cannot verify synthetic AddressBook local store: {err}"))?;
    if !store_exists {
        return Err("synthetic AddressBook local store row 0 does not exist".to_owned());
    }

    let transaction = connection
        .transaction_with_behavior(TransactionBehavior::Immediate)
        .map_err(|err| format!("cannot start synthetic AddressBook contact load: {err}"))?;
    load_exported_contacts(&transaction, export)?;
    transaction
        .commit()
        .map_err(|err| format!("cannot commit synthetic AddressBook contacts: {err}"))?;
    Ok(())
}

fn initialize_synthetic_store(connection: &Connection, source_hash: &str) -> Result<(), String> {
    let store: Option<i64> = connection
        .query_row(
            "SELECT ROWID FROM ABStore WHERE ROWID = 0 OR StoreInternalIdentifier = 'local'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|err| format!("cannot inspect synthetic AddressBook local store: {err}"))?;
    match store {
        Some(0) => Ok(()),
        Some(_) => {
            Err("the database already contains a local store, but it is not ROWID 0".to_owned())
        }
        None => {
            connection
                .execute(
                    "INSERT INTO ABStore(
                        ROWID, Type, StoreInternalIdentifier, AccountID, Enabled,
                        MeIdentifier, Capabilities, guid
                     ) VALUES (0, 0, 'local', -1, 0, -1, 0, ?1)",
                    [deterministic_guid(source_hash, "store:local")],
                )
                .map_err(|err| {
                    format!("cannot initialize synthetic AddressBook local store: {err}")
                })?;
            Ok(())
        }
    }
}

fn require_synthetic_tables(connection: &Connection) -> Result<(), String> {
    for table in [
        "ABPerson",
        "ABStore",
        "ABMultiValue",
        "ABMultiValueLabel",
        "ABPhoneLastFour",
    ] {
        let exists: bool = connection
            .query_row(
                "SELECT EXISTS(
                    SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = ?1
                 )",
                [table],
                |row| row.get(0),
            )
            .map_err(|err| format!("cannot inspect synthetic AddressBook table {table}: {err}"))?;
        if !exists {
            return Err(format!(
                "synthetic AddressBook schema is missing required table {table}"
            ));
        }
    }
    Ok(())
}

fn load_exported_contacts(
    transaction: &Transaction<'_>,
    export: &AddressBookExport,
) -> Result<(), String> {
    for contact in &export.contacts {
        let identity = format!("person:{}", contact.source.record_pk);
        let person_guid = deterministic_guid(&export.source.database_sha256, &identity);
        transaction
            .execute(
                "INSERT INTO ABPerson(First, Last, Middle, Organization, StoreID, guid)
                 VALUES (?1, ?2, ?3, ?4, 0, ?5)",
                rusqlite::params![
                    contact.name.first,
                    contact.name.last,
                    contact.name.middle,
                    contact.organization,
                    person_guid
                ],
            )
            .map_err(|err| {
                format!(
                    "cannot insert synthetic contact {}: {err}",
                    contact.source.record_pk
                )
            })?;
        let person_id = transaction.last_insert_rowid();

        for phone in &contact.phones {
            let multivalue_id = insert_synthetic_multivalue(
                transaction,
                person_id,
                &export.source.database_sha256,
                &identity,
                phone.source.row_pk,
                phone.value.as_deref(),
                phone.label.as_deref(),
                phone.ordering_index,
                3,
            )?;
            if phone.last_four.as_deref().is_some_and(is_four_ascii_digits) {
                transaction
                    .execute(
                        "INSERT INTO ABPhoneLastFour(multivalue_id, value) VALUES (?1, ?2)",
                        rusqlite::params![multivalue_id, phone.last_four],
                    )
                    .map_err(|err| {
                        format!("cannot insert synthetic phone last-four value: {err}")
                    })?;
            }
        }
        for email in &contact.emails {
            insert_synthetic_multivalue(
                transaction,
                person_id,
                &export.source.database_sha256,
                &identity,
                email.source.row_pk,
                email.value.as_deref(),
                email.label.as_deref(),
                email.ordering_index,
                4,
            )?;
        }
    }
    Ok(())
}

fn insert_synthetic_multivalue(
    connection: &Connection,
    record_id: i64,
    source_hash: &str,
    contact_identity: &str,
    source_row_pk: i64,
    value: Option<&str>,
    label: Option<&str>,
    ordering_index: Option<i64>,
    property_code: i64,
) -> Result<i64, String> {
    let value = value.filter(|value| !value.is_empty()).ok_or_else(|| {
        format!("multivalue for contact {contact_identity} must be a nonempty string")
    })?;
    let identifier = ordering_index.filter(|order| *order >= 0).ok_or_else(|| {
        format!("multivalue order for contact {contact_identity} must be nonnegative")
    })?;
    let label_id = match label.filter(|label| !label.is_empty()) {
        Some(label) => {
            connection
                .execute(
                    "INSERT OR IGNORE INTO ABMultiValueLabel(value) VALUES (?1)",
                    [label],
                )
                .map_err(|err| format!("cannot add synthetic AddressBook label: {err}"))?;
            Some(
                connection
                    .query_row(
                        "SELECT ROWID FROM ABMultiValueLabel WHERE value = ?1",
                        [label],
                        |row| row.get::<_, i64>(0),
                    )
                    .map_err(|err| format!("cannot resolve synthetic AddressBook label: {err}"))?,
            )
        }
        None => None,
    };
    let guid = deterministic_guid(
        source_hash,
        &format!("multivalue:{contact_identity}:{property_code}:{source_row_pk}"),
    );
    connection
        .execute(
            "INSERT INTO ABMultiValue(record_id, property, identifier, label, value, guid)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![record_id, property_code, identifier, label_id, value, guid],
        )
        .map_err(|err| format!("cannot insert synthetic AddressBook value: {err}"))?;
    Ok(connection.last_insert_rowid())
}

fn is_four_ascii_digits(value: &str) -> bool {
    value.len() == 4 && value.bytes().all(|byte| byte.is_ascii_digit())
}

fn deterministic_guid(source_hash: &str, identity: &str) -> String {
    let namespace = Uuid::from_bytes([
        0x52, 0xac, 0x16, 0x9a, 0x51, 0x44, 0x4a, 0x54, 0xa6, 0xd8, 0x99, 0xfe, 0x9d, 0xe0, 0xca,
        0x95,
    ]);
    Uuid::new_v5(&namespace, format!("{source_hash}:{identity}").as_bytes())
        .to_string()
        .to_uppercase()
}

fn copy_verified_tree(
    source_root: &Path,
    manifest_prefix: &Path,
    destination_root: &Path,
    manifest_hashes: &HashMap<PathBuf, String>,
) -> Result<(), String> {
    if !source_root.is_dir() {
        return Err(format!(
            "evidence directory does not exist: {}",
            source_root.display()
        ));
    }

    fn copy_directory(
        source_dir: &Path,
        source_root: &Path,
        manifest_prefix: &Path,
        destination_dir: &Path,
        manifest_hashes: &HashMap<PathBuf, String>,
    ) -> Result<(), String> {
        fs::create_dir_all(destination_dir).map_err(|err| {
            format!(
                "cannot create Cellebrite directory '{}': {}",
                destination_dir.display(),
                err
            )
        })?;

        let entries = fs::read_dir(source_dir).map_err(|err| {
            format!(
                "cannot read evidence directory '{}': {}",
                source_dir.display(),
                err
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|err| {
                format!(
                    "cannot read an entry in evidence directory '{}': {}",
                    source_dir.display(),
                    err
                )
            })?;
            let source_path = entry.path();
            let file_type = entry.file_type().map_err(|err| {
                format!(
                    "cannot inspect evidence path '{}': {}",
                    source_path.display(),
                    err
                )
            })?;

            if file_type.is_dir() {
                copy_directory(
                    &source_path,
                    source_root,
                    manifest_prefix,
                    &destination_dir.join(entry.file_name()),
                    manifest_hashes,
                )?;
            } else if file_type.is_file() {
                let relative_path = source_path.strip_prefix(source_root).map_err(|err| {
                    format!(
                        "evidence path '{}' is not beneath '{}': {}",
                        source_path.display(),
                        source_root.display(),
                        err
                    )
                })?;
                let manifest_path = manifest_prefix.join(relative_path);
                let expected_hash = manifest_hashes.get(&manifest_path).ok_or_else(|| {
                    format!(
                        "file '{}' is missing from hash manifest",
                        manifest_path.display()
                    )
                })?;
                let destination_path = destination_dir.join(entry.file_name());
                fs::copy(&source_path, &destination_path).map_err(|err| {
                    format!(
                        "cannot copy evidence file '{}' to '{}': {}",
                        source_path.display(),
                        destination_path.display(),
                        err
                    )
                })?;
                let actual_hash = calculate_sha256(&destination_path)?;
                if !actual_hash.eq_ignore_ascii_case(expected_hash) {
                    return Err(format!(
                        "SHA-256 mismatch for '{}': expected {}, got {}",
                        manifest_path.display(),
                        expected_hash,
                        actual_hash
                    ));
                }
            }
        }

        Ok(())
    }

    copy_directory(
        source_root,
        source_root,
        manifest_prefix,
        destination_root,
        manifest_hashes,
    )
}

pub fn parse_hash_manifest_line(
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
    permissions.set_mode(permissions.mode() & !0o131313);
    fs::set_permissions(path, permissions)
}

pub fn archive_evidence(source_dir: &Path, archive_path: &Path) -> Result<(), String> {
    let archive_parent = archive_path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));

    fs::create_dir_all(archive_parent).map_err(|err| {
        format!(
            "cannot create parent directory for archive '{}': {}",
            archive_parent.display(),
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

    set_read_only(archive_path).expect("setting vault ZIP to read-only");

    Ok(())
}
