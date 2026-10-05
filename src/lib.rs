use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::io::{Write, Read};
use sha2::{Sha256, Digest};
use hex;

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
    let mut file = fs::File::open(path).map_err(|e| format!("cannot open file '{}': {}", path.display(), e))?;
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

/// Recursively finds regular files beneath `home_dir` and returns their relative paths.
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

    let mut evidence_files = Vec::new();
    visit_directory(home_dir, home_dir, &mut evidence_files)?;
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
    let out_file_path = output_dir.join("evidence").join("vault").join("hash_manifest.txt");
    let mut manifest_entries: Vec<(String, String)> = Vec::new();

    // recursively scan the home_dir and get relative paths
    let evidence_files = find_evidence_files(home_dir)?;

    println!("Writing evidence hashes to output directory: {}", out_file_path.display());
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