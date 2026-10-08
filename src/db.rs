use rusqlite::Connection;
use std::path::Path;

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

pub fn get_all_addresses(
    manifest_path: &std::path::Path,
    working_dir: &std::path::Path,
) -> Result<String, String> {
    use crate::evidence::parse_hash_manifest_line;
    use std::fs;

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

pub fn get_all_messages(chat_db: &Path) -> Result<String, String> {
    #[derive(serde::Serialize)]
    struct Message {
        timestamp: Option<String>,
        chat: Option<String>,
        sender: Option<String>,
        is_from_me: Option<bool>,
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
                is_from_me: row.get(3)?,
                message: row.get(4)?,
                attachment: row.get(5)?,
            })
        })
        .map_err(|err| format!("cannot query messages: {}", err))?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| format!("cannot read message row: {}", err))?;

    serde_json::to_string(&messages)
        .map_err(|err| format!("cannot serialize messages as JSON: {}", err))
}
