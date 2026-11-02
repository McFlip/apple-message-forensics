# Database Schema and SQL Queries Reference

Technical reference for the source SQLite databases and the SQL extraction queries executed by `apple-message-forensics`.

## Source Database Locations

On macOS, source files reside in the user's Library folder:

| Source Artifact | Default macOS Path | Purpose |
|---|---|---|
| Messages Database | `~/Library/Messages/chat.db` | Primary message, handle, and chat database. |
| Messages WAL Sidecar | `~/Library/Messages/chat.db-wal` | Active uncommitted SQLite write-ahead log. |
| Messages SHM Sidecar | `~/Library/Messages/chat.db-shm` | Shared memory index for the WAL file. |
| Attachments | `~/Library/Messages/Attachments/` | Stored image, video, audio, and file payload vaults. |
| AddressBook Database | `~/Library/Application Support/AddressBook/AddressBook-v22.abcddb` | macOS local contact directory. |
| Containerized AddressBook | `~/Library/Application Support/AddressBook/Sources/<GUID>/...` | Synchronized cloud accounts (e.g. iCloud, CardDAV). |

## SQL Query Definitions

The extraction engine relies on three primary SQL queries executed against the preserved SQLite databases.

### 1. Handles Query (`query/handle-query.sql`)

Extracts unique participant identifiers from `chat.db`:

```sql
SELECT
    ROWID,
    id,
    country,
    service,
    uncanonicalized_id
FROM handle;
```

- **`ROWID`**: Integer primary key joined against `chat_message_join` and `chat_handle_join`.
- **`id`**: E.164 phone number, Apple Account email address, or service identifier.
- **`service`**: Identifier (`iMessage`, `SMS`).

### 2. Messages Query (`query/msg-query.sql`)

Extracts raw message records, join references, and attachment references from `chat.db`:

```sql
SELECT
    m.ROWID as message_id,
    m.guid as message_guid,
    m.text as text,
    m.date as date,
    m.date_read as date_read,
    m.date_delivered as date_delivered,
    m.is_from_me as is_from_me,
    m.handle_id as handle_id,
    m.service as service,
    c.ROWID as chat_id,
    c.guid as chat_guid,
    c.chat_identifier as chat_identifier,
    c.display_name as chat_display_name,
    a.ROWID as attachment_id,
    a.guid as attachment_guid,
    a.filename as attachment_filename,
    a.mime_type as attachment_mime_type,
    a.total_bytes as attachment_total_bytes
FROM message m
LEFT JOIN chat_message_join cmj ON m.ROWID = cmj.message_id
LEFT JOIN chat c ON cmj.chat_id = c.ROWID
LEFT JOIN message_attachment_join maj ON m.ROWID = maj.message_id
LEFT JOIN attachment a ON maj.attachment_id = a.ROWID
ORDER BY m.date ASC;
```

**Timestamp Encoding Note:** SQLite stores timestamps as nanoseconds since Apple Cocoa epoch (`2001-01-01 00:00:00 UTC`). The tool converts these values to Unix epoch before rendering RFC 3339 representations.

### 3. AddressBook Query (`query/addr-query.sql`)

Extracts names, emails, and phone numbers from `AddressBook-v22.abcddb`:

```sql
SELECT
    r.Z_PK as record_id,
    r.ZUNIQUEID as contact_unique_id,
    r.ZFIRSTNAME as first_name,
    r.ZLASTNAME as last_name,
    r.ZORGANIZATION as organization,
    p.ZFULLNUMBER as phone_number,
    e.ZADDRESS as email_address
FROM ZABCDRECORD r
LEFT JOIN ZABCDPHONENUMBER p ON r.Z_PK = p.ZOWNER
LEFT JOIN ZABCDEMAILADDRESS e ON r.Z_PK = e.ZOWNER;
```

## Schema References in Repository

Complete DDL dumps for reference and verification are maintained under `schema/`:

- `schema/chat-schema.sql`: Full DDL dump of Apple Messages `chat.db`.
- `schema/addr-schema.sql`: Full DDL dump of macOS `AddressBook-v22.abcddb`.
- `schema/iPhoneSyntheticSchema.sql`: Reference schema matching iOS `sms.db`.
