use rusqlite::{Connection, OpenFlags};
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Deserialize)]
struct AddressBookContainers {
    containers: Vec<AddressBookContainer>,
}

#[derive(Deserialize)]
struct AddressBookContainer {
    #[serde(rename = "isICloudCandidate")]
    is_icloud_candidate: bool,
    #[serde(rename = "sourceUUID")]
    source_uuid: Option<String>,
    #[serde(rename = "sourcePath")]
    source_path: Option<PathBuf>,
}

pub fn discover_address_books(output_dir: &Path) -> Result<PathBuf, String> {
    let output = Command::new("swift")
        .arg("scripts/list-containers.swift")
        .output()
        .map_err(|error| format!("failed to run Address Book discovery script: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        return Err(if stderr.is_empty() {
            format!(
                "Address Book discovery script exited with status {}",
                output.status
            )
        } else {
            format!("Address Book discovery script failed: {stderr}")
        });
    }

    let json_dir = output_dir.join("json");
    std::fs::create_dir_all(&json_dir)
        .map_err(|error| format!("cannot create {}: {error}", json_dir.display()))?;
    let output_path = json_dir.join("contact-containers.json");
    std::fs::write(&output_path, output.stdout)
        .map_err(|error| format!("cannot write {}: {error}", output_path.display()))?;

    Ok(output_path)
}

pub fn find_icloud_address_book_uuid(containers_json: &Path) -> Result<String, String> {
    let json = std::fs::read_to_string(containers_json)
        .map_err(|error| format!("cannot read {}: {error}", containers_json.display()))?;
    let address_books: AddressBookContainers = serde_json::from_str(&json)
        .map_err(|error| format!("cannot parse Address Book containers JSON: {error}"))?;
    let mut candidates = address_books
        .containers
        .into_iter()
        .filter(|container| container.is_icloud_candidate);

    let candidate = candidates.next().ok_or_else(|| {
        "No iCloud Address Book container found. Remember to sign in to iCloud and sync from the target phone.".to_owned()
    })?;

    if candidates.next().is_some() {
        return Err("More than one iCloud Address Book container was found.".to_owned());
    }

    let uuid = candidate
        .source_uuid
        .ok_or_else(|| "The iCloud Address Book container has no source UUID.".to_owned())?;
    let source_path = candidate
        .source_path
        .ok_or_else(|| "The iCloud Address Book container has no source path.".to_owned())?;
    let database_path = source_path.join("AddressBook-v22.abcddb");

    if !database_path.is_file() {
        return Err(format!(
            "iCloud Address Book database does not exist: {}",
            database_path.display()
        ));
    }

    let connection = Connection::open_with_flags(&database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("cannot open iCloud Address Book database: {error}"))?;
    let integrity: String = connection
        .query_row("PRAGMA integrity_check", [], |row| row.get(0))
        .map_err(|error| format!("iCloud Address Book database integrity check failed: {error}"))?;

    if integrity != "ok" {
        return Err(format!(
            "iCloud Address Book database integrity check failed: {integrity}"
        ));
    }

    Ok(uuid)
}
