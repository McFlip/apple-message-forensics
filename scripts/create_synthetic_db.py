#!/usr/bin/env python3
"""Create and validate a synthetic iPhone Address Book SQLite database."""

from pathlib import Path
import sqlite3


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


if __name__ == "__main__":
    create_synthetic_database(
        schema_path=Path("iPhoneSyntheticSchema.sql"),
        database_path=Path("iphone_synthetic.db"),
    )