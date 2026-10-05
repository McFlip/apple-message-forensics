use hex;
use rusqlite::Connection;
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    thread,
    time::Duration,
};
use std::os::unix::fs::PermissionsExt;

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