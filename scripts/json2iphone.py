#!/usr/bin/env python3
"""Create and validate a synthetic iPhone Address Book SQLite database."""

from __future__ import annotations
import argparse
import json
import re
import sqlite3
import uuid
from pathlib import Path
from typing import Any

PHONE_PROPERTY = 3
EMAIL_PROPERTY = 4
GUID_NAMESPACE = uuid.UUID("52ac169a-5144-4a54-a6d8-99fe9de0ca95")
FOUR_DIGITS = re.compile(r"^[0-9]{4}$")

def create_synthetic_database(
    schema_path: Path,
    database_path: Path,
) -> None:
    """Create a new synthetic database from the supplied SQL schema."""

    if database_path.exists():
        raise FileExistsError(
            f"Refusing to overwrite existing database: {database_path}"
        )

    schema_sql = schema_path.read_text(encoding="utf-8")

    with sqlite3.connect(database_path) as connection:
        connection.executescript(schema_sql)

        integrity_result = connection.execute(
            "PRAGMA integrity_check"
        ).fetchone()[0]

        if integrity_result != "ok":
            raise RuntimeError(
                f"SQLite integrity check failed: {integrity_result}"
            )

        tables = connection.execute(
            """
            SELECT name
            FROM sqlite_master
            WHERE type = 'table'
            ORDER BY name
            """
        ).fetchall()

    print(f"Created: {database_path}")
    print("Integrity check: ok")
    print("Tables:")
    for table in tables:
        print(f"  {table[0]}")

def initialize_synthetic_local_store(
    connection: sqlite3.Connection,
    store_guid: str,
) -> None:
    """Create the minimum local store required by a schema-only forensic target."""

    existing_store = connection.execute(
        """
        SELECT ROWID, StoreInternalIdentifier
        FROM ABStore
        WHERE ROWID = 0 OR StoreInternalIdentifier = 'local'
        """
    ).fetchone()

    if existing_store is None:
        connection.execute(
            """
            INSERT INTO ABStore(
                ROWID,
                Type,
                StoreInternalIdentifier,
                AccountID,
                Enabled,
                MeIdentifier,
                Capabilities,
                guid
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            """,
            (
                0,
                0,
                "local",
                -1,
                0,
                -1,
                0,
                store_guid,
            ),
        )
        return

    if existing_store[0] != 0:
        raise RuntimeError(
            "The database already contains a local store, but it is not ROWID 0."
        )

def deterministic_guid(source_hash: str, identity: str) -> str:
    """Create a stable synthetic target identifier without reusing source Apple identifiers."""

    return str(
        uuid.uuid5(
            GUID_NAMESPACE,
            f"{source_hash}:{identity}",
        )
    ).upper()


def require_target_tables(connection: sqlite3.Connection) -> None:
    """Ensure that the destination exposes the minimum iPhone contact tables."""

    required_tables = {
        "ABPerson",
        "ABStore",
        "ABMultiValue",
        "ABMultiValueLabel",
        "ABPhoneLastFour",
    }
    actual_tables = {
        row[0]
        for row in connection.execute(
            "SELECT name FROM sqlite_master WHERE type = 'table'"
        )
    }
    missing_tables = required_tables - actual_tables

    if missing_tables:
        missing = ", ".join(sorted(missing_tables))
        raise RuntimeError(f"Destination is not a compatible iPhone template: {missing}")


def require_empty_destination(connection: sqlite3.Connection) -> None:
    """Reject a nonempty destination to prevent accidental merges or duplicate contacts."""

    person_count = connection.execute(
        "SELECT COUNT(*) FROM ABPerson"
    ).fetchone()[0]
    multivalue_count = connection.execute(
        "SELECT COUNT(*) FROM ABMultiValue"
    ).fetchone()[0]

    if person_count or multivalue_count:
        raise RuntimeError(
            "Destination must be empty; create a fresh template copy for each load."
        )


def require_store(connection: sqlite3.Connection, store_id: int) -> None:
    """Ensure that the selected destination contact store already exists."""

    exists = connection.execute(
        "SELECT 1 FROM ABStore WHERE ROWID = ?",
        (store_id,),
    ).fetchone()

    if exists is None:
        raise RuntimeError(f"ABStore row {store_id} does not exist in the template.")


def label_id(
    connection: sqlite3.Connection,
    label_value: str | None,
) -> int | None:
    """Resolve a target label reference while preserving nonblank source label text."""

    if label_value is None or label_value == "":
        return None

    connection.execute(
        "INSERT OR IGNORE INTO ABMultiValueLabel(value) VALUES (?)",
        (label_value,),
    )

    row = connection.execute(
        "SELECT ROWID FROM ABMultiValueLabel WHERE value = ?",
        (label_value,),
    ).fetchone()

    if row is None:
        raise RuntimeError(f"Unable to resolve label: {label_value!r}")

    return int(row[0])


def validate_value(value: Any, field_name: str) -> str:
    """Reject unsupported semantic values rather than silently changing or omitting them."""

    if not isinstance(value, str) or value == "":
        raise ValueError(f"{field_name} must be a nonempty string.")

    return value


def validate_order(order: Any, field_name: str) -> int:
    """Validate the source ordering value used as the iPhone multivalue identifier."""

    if not isinstance(order, int) or order < 0:
        raise ValueError(f"{field_name} must be a nonnegative integer.")

    return order


def insert_multivalue(
    connection: sqlite3.Connection,
    record_id: int,
    source_hash: str,
    contact_identity: str,
    value: dict[str, Any],
    property_code: int,
) -> int:
    """Insert one phone or email value and return its target multivalue identifier."""

    raw_value = validate_value(value["value"], "multivalue.value")
    identifier = validate_order(value["order"], "multivalue.order")
    target_label = label_id(connection, value["label"])
    source_row_pk = value["source"]["row_pk"]
    target_guid = deterministic_guid(
        source_hash,
        f"multivalue:{contact_identity}:{property_code}:{source_row_pk}",
    )

    cursor = connection.execute(
        """
        INSERT INTO ABMultiValue(
            record_id,
            property,
            identifier,
            label,
            value,
            guid
        )
        VALUES (?, ?, ?, ?, ?, ?)
        """,
        (
            record_id,
            property_code,
            identifier,
            target_label,
            raw_value,
            target_guid,
        ),
    )

    return int(cursor.lastrowid)


def load_contacts(
    destination_path: Path,
    payload: dict[str, Any],
    store_id: int,
) -> dict[str, int]:
    """Load supported canonical JSON contact data into an empty iPhone database template."""

    source_hash = payload["source"]["database_sha256"]
    contacts = payload["contacts"]
    connection = sqlite3.connect(destination_path)

    try:
        require_target_tables(connection)
        require_empty_destination(connection)
        require_store(connection, store_id)

        connection.execute("BEGIN IMMEDIATE")

        try:
            phone_count = 0
            email_count = 0

            for contact in contacts:
                record_pk = contact["source"]["record_pk"]
                contact_identity = f"person:{record_pk}"
                name = contact["name"]

                person_guid = deterministic_guid(
                    source_hash,
                    contact_identity,
                )

                person_cursor = connection.execute(
                    """
                    INSERT INTO ABPerson(
                        First,
                        Last,
                        Middle,
                        Organization,
                        StoreID,
                        guid
                    )
                    VALUES (?, ?, ?, ?, ?, ?)
                    """,
                    (
                        name["first"],
                        name["last"],
                        name["middle"],
                        contact["organization"],
                        store_id,
                        person_guid,
                    ),
                )

                person_id = int(person_cursor.lastrowid)

                for phone in contact["phones"]:
                    multivalue_id = insert_multivalue(
                        connection,
                        person_id,
                        source_hash,
                        contact_identity,
                        phone,
                        PHONE_PROPERTY,
                    )
                    last_four = phone["last_four"]

                    if isinstance(last_four, str) and FOUR_DIGITS.fullmatch(last_four):
                        connection.execute(
                            """
                            INSERT INTO ABPhoneLastFour(multivalue_id, value)
                            VALUES (?, ?)
                            """,
                            (multivalue_id, last_four),
                        )

                    phone_count += 1

                for email in contact["emails"]:
                    insert_multivalue(
                        connection,
                        person_id,
                        source_hash,
                        contact_identity,
                        email,
                        EMAIL_PROPERTY,
                    )
                    email_count += 1

            connection.commit()

            return {
                "contacts": len(contacts),
                "phones": phone_count,
                "emails": email_count,
            }
        except Exception:
            connection.rollback()
            raise
    finally:
        connection.close()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Load canonical contact JSON into a synthetic iPhone Address Book."
    )
    parser.add_argument("--input-json", required=True, type=Path)
    parser.add_argument("--schema", required=True, type=Path)
    parser.add_argument("--output-database", required=True, type=Path)
    arguments = parser.parse_args()

    create_synthetic_database(
        schema_path=arguments.schema,
        database_path=arguments.output_database,
    )

    with arguments.input_json.open("r", encoding="utf-8") as stream:
        contact_payload = json.load(stream)

    with sqlite3.connect(arguments.output_database) as template_connection:
        initialize_synthetic_local_store(
            template_connection,
            deterministic_guid(
                contact_payload["source"]["database_sha256"],
                "store:local",
            ),
        )

    report = load_contacts(
        destination_path=arguments.output_database,
        payload=contact_payload,
        store_id=0,
    )

    print(json.dumps(report, indent=2, sort_keys=True))
