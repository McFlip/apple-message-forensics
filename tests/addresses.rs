use apple_message_forensics::get_all_addresses;
use rusqlite::Connection;
use serde_json::json;
use std::{fs, path::Path};
use tempfile::tempdir;

const ADDRESS_BOOK_SCHEMA: &str = include_str!("../schema/addr-schema.sql");
const ADDRESS_BOOK_PATHS: [&str; 2] = [
    "Library/Application Support/AddressBook/AddressBook-v22.abcddb",
    "Library/Application Support/AddressBook/Sources/E35DB509-AB16-4698-BE02-B7883112E9B0/AddressBook-v22.abcddb",
];
type AddressBookRecord = (
    &'static str,
    &'static str,
    Option<&'static str>,
    &'static str,
    Option<&'static str>,
    Option<&'static str>,
    Option<&'static str>,
    &'static str,
    &'static str,
);

#[test]
fn get_all_addresses_combines_contacts_from_all_manifest_address_books() {
    let temp_dir = tempdir().expect("create temporary directory");
    let working_dir = temp_dir.path().join("working_copy");
    fs::create_dir_all(&working_dir).expect("create working-copy directory");

    let first_address_book = [
        (
            "record-ada",
            "Ada",
            Some("Byron"),
            "Lovelace",
            Some("Ada"),
            Some("Analytical Engines"),
            Some("Mathematician"),
            "+1-555-0101",
            "ada@example.test",
        ),
        (
            "record-grace",
            "Grace",
            None,
            "Hopper",
            Some("Amazing Grace"),
            Some("Navy"),
            Some("Rear Admiral"),
            "+1-555-0102",
            "grace@example.test",
        ),
        (
            "record-alan",
            "Alan",
            Some("Mathison"),
            "Turing",
            None,
            Some("Bletchley Park"),
            Some("Cryptanalyst"),
            "+1-555-0103",
            "alan@example.test",
        ),
    ];
    let second_address_book = [
        (
            "record-barbara",
            "Barbara",
            Some("Mary"),
            "Liskov",
            None,
            Some("MIT"),
            Some("Computer Scientist"),
            "+1-555-0201",
            "barbara@example.test",
        ),
        (
            "record-katherine",
            "Katherine",
            Some("Coleman"),
            "Johnson",
            Some("Katherine"),
            Some("NASA"),
            Some("Mathematician"),
            "+1-555-0202",
            "katherine@example.test",
        ),
        (
            "record-dorothy",
            "Dorothy",
            Some("Jean"),
            "Vaughan",
            None,
            Some("NASA"),
            Some("Mathematician"),
            "+1-555-0203",
            "dorothy@example.test",
        ),
    ];

    create_address_book(
        &working_dir.join(ADDRESS_BOOK_PATHS[0]),
        &first_address_book,
    );
    create_address_book(
        &working_dir.join(ADDRESS_BOOK_PATHS[1]),
        &second_address_book,
    );

    let manifest_path = temp_dir.path().join("hash_manifest.txt");
    let manifest = ADDRESS_BOOK_PATHS
        .iter()
        .map(|path| format!("{}  {path}\n", "0".repeat(64)))
        .collect::<String>();
    fs::write(&manifest_path, manifest).expect("write address-book manifest");

    let actual = get_all_addresses(&manifest_path, &working_dir).expect("get all addresses");
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
        },
        {
            "contact_id": 2,
            "contact_unique_id": "record-katherine",
            "record_type": 0,
            "first_name": "Katherine",
            "middle_name": "Coleman",
            "last_name": "Johnson",
            "nickname": "Katherine",
            "organization": "NASA",
            "job_title": "Mathematician",
            "phone_numbers": "+1-555-0202",
            "email_addresses": "katherine@example.test"
        },
        {
            "contact_id": 1,
            "contact_unique_id": "record-barbara",
            "record_type": 0,
            "first_name": "Barbara",
            "middle_name": "Mary",
            "last_name": "Liskov",
            "nickname": null,
            "organization": "MIT",
            "job_title": "Computer Scientist",
            "phone_numbers": "+1-555-0201",
            "email_addresses": "barbara@example.test"
        },
        {
            "contact_id": 3,
            "contact_unique_id": "record-dorothy",
            "record_type": 0,
            "first_name": "Dorothy",
            "middle_name": "Jean",
            "last_name": "Vaughan",
            "nickname": null,
            "organization": "NASA",
            "job_title": "Mathematician",
            "phone_numbers": "+1-555-0203",
            "email_addresses": "dorothy@example.test"
        }
    ]);

    assert_eq!(actual_json, expected_json);
}

fn create_address_book(
    address_book_path: &Path,
    records: &[AddressBookRecord; 3],
) {
    fs::create_dir_all(
        address_book_path
            .parent()
            .expect("address book path should have parent"),
    )
    .expect("create address-book directory");

    let connection = Connection::open(address_book_path).expect("create address-book database");
    connection
        .execute_batch(ADDRESS_BOOK_SCHEMA)
        .expect("initialize address-book schema");

    for (index, record) in records.iter().enumerate() {
        let contact_id = (index + 1) as i64;
        connection
            .execute(
                "INSERT INTO ZABCDRECORD (
                    Z_PK, ZUNIQUEID, ZTYPE, ZFIRSTNAME, ZMIDDLENAME, ZLASTNAME,
                    ZNICKNAME, ZORGANIZATION, ZJOBTITLE
                ) VALUES (?1, ?2, 0, ?3, ?4, ?5, ?6, ?7, ?8)",
                (
                    contact_id, record.0, record.1, record.2, record.3, record.4, record.5,
                    record.6,
                ),
            )
            .expect("insert address-book contact");
        connection
            .execute(
                "INSERT INTO ZABCDPHONENUMBER (Z_PK, ZOWNER, ZFULLNUMBER)
                 VALUES (?1, ?2, ?3)",
                (contact_id, contact_id, record.7),
            )
            .expect("insert address-book phone number");
        connection
            .execute(
                "INSERT INTO ZABCDEMAILADDRESS (Z_PK, ZOWNER, ZADDRESS)
                 VALUES (?1, ?2, ?3)",
                (contact_id, contact_id, record.8),
            )
            .expect("insert address-book email address");
    }
}
