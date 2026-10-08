mod common;

use apple_message_forensics::get_all_handles;
use common::create_test_chat_db;

#[test]
fn get_all_handles_returns_handles_as_json() {
    let (temp_dir, connection) = create_test_chat_db();
    let db_path = temp_dir.path().join("chat.db");

    connection
        .execute(
            "INSERT INTO handle (ROWID, id, country, service, uncanonicalized_id, person_centric_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                "1",
                "+15550000001",
                "US",
                "iMessage",
                "+1 (555) 000-0001",
                "person-1",
            ),
        )
        .expect("insert first handle");
    connection
        .execute(
            "INSERT INTO handle (ROWID, id, country, service, uncanonicalized_id, person_centric_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                "2",
                "+15550000002",
                "US",
                "iMessage",
                "+1 (555) 000-0002",
                "person-2",
            ),
        )
        .expect("insert second handle");
    connection
        .execute(
            "INSERT INTO handle (ROWID, id, country, service, uncanonicalized_id, person_centric_id)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            (
                "3",
                "alice@example.com",
                "GB",
                "SMS",
                "alice@example.com",
                "person-3",
            ),
        )
        .expect("insert third handle");

    let expected = r#"[{"handle_id":1,"handle_identifier":"+15550000001","service":"iMessage","country":"US","uncanonicalized_id":"+1 (555) 000-0001","person_centric_id":"person-1"},{"handle_id":2,"handle_identifier":"+15550000002","service":"iMessage","country":"US","uncanonicalized_id":"+1 (555) 000-0002","person_centric_id":"person-2"},{"handle_id":3,"handle_identifier":"alice@example.com","service":"SMS","country":"GB","uncanonicalized_id":"alice@example.com","person_centric_id":"person-3"}]"#;

    let actual = get_all_handles(&db_path).expect("get handles");

    assert_eq!(actual, expected);
}
