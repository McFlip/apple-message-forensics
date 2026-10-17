#!/usr/bin/env python3
"""Export selected macOS Apple Address Book contact data to forensic JSON."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sqlite3
from pathlib import Path
from typing import Any


def sha256_file(path: Path) -> str:
    """Return the SHA-256 digest of an input artifact."""

    digest = hashlib.sha256()

    with path.open("rb") as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(block)

    return digest.hexdigest()


def open_read_only_database(path: Path) -> sqlite3.Connection:
    """Open an evidence copy without permitting SQLite writes or journal recovery changes."""

    uri = f"{path.resolve().as_uri()}?mode=ro&immutable=1"
    connection = sqlite3.connect(uri, uri=True)
    connection.row_factory = sqlite3.Row
    connection.execute("PRAGMA query_only = ON")

    return connection


def require_columns(
    connection: sqlite3.Connection,
    table_name: str,
    required_columns: set[str],
) -> None:
    """Ensure that the source database exposes the fields required by this exporter."""

    available_columns = {
        row["name"]
        for row in connection.execute(f"PRAGMA table_info({table_name})")
    }
    missing_columns = required_columns - available_columns

    if missing_columns:
        missing = ", ".join(sorted(missing_columns))
        raise RuntimeError(f"{table_name} is missing required columns: {missing}")


def fetch_phone_values(
    connection: sqlite3.Connection,
    record_pk: int,
) -> list[dict[str, Any]]:
    """Return phone values in their original source ordering for one contact."""

    rows = connection.execute(
        """
        SELECT
            Z_PK AS row_pk,
            ZUNIQUEID AS row_uid,
            ZFULLNUMBER AS value,
            ZLABEL AS label,
            ZORDERINGINDEX AS ordering_index,
            ZISPRIMARY AS is_primary,
            ZLASTFOURDIGITS AS last_four
        FROM ZABCDPHONENUMBER
        WHERE ZOWNER = ?
        ORDER BY ZORDERINGINDEX, Z_PK
        """,
        (record_pk,),
    )

    return [
        {
            "source": {
                "row_pk": row["row_pk"],
                "row_uid": row["row_uid"],
                "is_primary": bool(row["is_primary"])
                if row["is_primary"] is not None
                else None,
            },
            "value": row["value"],
            "label": row["label"],
            "order": row["ordering_index"],
            "last_four": row["last_four"],
        }
        for row in rows
    ]


def fetch_email_values(
    connection: sqlite3.Connection,
    record_pk: int,
) -> list[dict[str, Any]]:
    """Return email values in their original source ordering for one contact."""

    rows = connection.execute(
        """
        SELECT
            Z_PK AS row_pk,
            ZUNIQUEID AS row_uid,
            ZADDRESS AS value,
            ZADDRESSNORMALIZED AS normalized_value,
            ZLABEL AS label,
            ZORDERINGINDEX AS ordering_index,
            ZISPRIMARY AS is_primary
        FROM ZABCDEMAILADDRESS
        WHERE ZOWNER = ?
        ORDER BY ZORDERINGINDEX, Z_PK
        """,
        (record_pk,),
    )

    return [
        {
            "source": {
                "row_pk": row["row_pk"],
                "row_uid": row["row_uid"],
                "is_primary": bool(row["is_primary"])
                if row["is_primary"] is not None
                else None,
            },
            "value": row["value"],
            "normalized_value": row["normalized_value"],
            "label": row["label"],
            "order": row["ordering_index"],
        }
        for row in rows
    ]


def export_mac_contacts(source_path: Path) -> dict[str, Any]:
    """Build a canonical forensic JSON document from a macOS Address Book database."""

    connection = open_read_only_database(source_path)

    try:
        require_columns(
            connection,
            "ZABCDRECORD",
            {
                "Z_PK",
                "ZUNIQUEID",
                "ZFIRSTNAME",
                "ZMIDDLENAME",
                "ZLASTNAME",
                "ZORGANIZATION",
            },
        )
        require_columns(
            connection,
            "ZABCDPHONENUMBER",
            {
                "Z_PK",
                "ZUNIQUEID",
                "ZOWNER",
                "ZFULLNUMBER",
                "ZLABEL",
                "ZORDERINGINDEX",
                "ZISPRIMARY",
                "ZLASTFOURDIGITS",
            },
        )
        require_columns(
            connection,
            "ZABCDEMAILADDRESS",
            {
                "Z_PK",
                "ZUNIQUEID",
                "ZOWNER",
                "ZADDRESS",
                "ZADDRESSNORMALIZED",
                "ZLABEL",
                "ZORDERINGINDEX",
                "ZISPRIMARY",
            },
        )

        records = connection.execute(
            """
            SELECT
                r.Z_PK AS record_pk,
                r.ZUNIQUEID AS record_uid,
                r.ZFIRSTNAME AS first_name,
                r.ZMIDDLENAME AS middle_name,
                r.ZLASTNAME AS last_name,
                r.ZORGANIZATION AS organization
            FROM ZABCDRECORD AS r
            WHERE
                r.ZFIRSTNAME IS NOT NULL
                OR r.ZMIDDLENAME IS NOT NULL
                OR r.ZLASTNAME IS NOT NULL
                OR r.ZORGANIZATION IS NOT NULL
                OR EXISTS (
                    SELECT 1
                    FROM ZABCDPHONENUMBER AS p
                    WHERE p.ZOWNER = r.Z_PK
                )
                OR EXISTS (
                    SELECT 1
                    FROM ZABCDEMAILADDRESS AS e
                    WHERE e.ZOWNER = r.Z_PK
                )
            ORDER BY r.Z_PK
            """
        )

        contacts = [
            {
                "source": {
                    "record_pk": record["record_pk"],
                    "record_uid": record["record_uid"],
                },
                "name": {
                    "first": record["first_name"],
                    "middle": record["middle_name"],
                    "last": record["last_name"],
                },
                "organization": record["organization"],
                "phones": fetch_phone_values(connection, record["record_pk"]),
                "emails": fetch_email_values(connection, record["record_pk"]),
            }
            for record in records
        ]

        return {
            "format": "apple-addressbook-forensic-v1",
            "source": {
                "database_sha256": sha256_file(source_path),
                "platform": "macOS",
                "tables": [
                    "ZABCDRECORD",
                    "ZABCDPHONENUMBER",
                    "ZABCDEMAILADDRESS",
                ],
            },
            "contacts": contacts,
            "target_losses": [
                "Mac primary-value flags are preserved in JSON but have no direct ABMultiValue equivalent.",
                "Mac record and child unique IDs are preserved as source provenance only.",
                "Images, postal addresses, sync metadata, account metadata, history, and search artifacts are omitted.",
            ],
        }
    finally:
        connection.close()


def write_json_atomically(payload: dict[str, Any], output_path: Path) -> None:
    """Write JSON deterministically without leaving a partial output artifact."""

    temporary_path = output_path.with_suffix(f"{output_path.suffix}.tmp")

    with temporary_path.open("w", encoding="utf-8", newline="\n") as stream:
        json.dump(
            payload,
            stream,
            ensure_ascii=False,
            indent=2,
            sort_keys=True,
        )
        stream.write("\n")
        stream.flush()
        os.fsync(stream.fileno())

    os.replace(temporary_path, output_path)


if __name__ == "__main__":
    parser = argparse.ArgumentParser(
        description="Export selected macOS Address Book contact data to JSON."
    )
    parser.add_argument("--source", required=True, type=Path)
    parser.add_argument("--output", required=True, type=Path)
    arguments = parser.parse_args()

    arguments.output.parent.mkdir(parents=True, exist_ok=True)

    write_json_atomically(
        export_mac_contacts(arguments.source),
        arguments.output,
    )