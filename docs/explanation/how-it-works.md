# How it Works: Architecture and Evidence Lineage

This document explains how `apple-message-forensics` operates under the hood, how Apple manages messages and address books on macOS, and the end-to-end lineage of preserved data.

## Evidence Lineage Overview

When providing testimony or submitting a digital forensics report, understanding how data transitions from raw operating system artifacts to delivered work product is paramount:

```text
Host System Artifacts
  ├── Messages Database (chat.db, chat.db-wal, chat.db-shm)
  ├── Message Attachments Directory
  └── AddressBook / Contacts Databases
              │
              ▼
    [1. Evidence Acquisition]
  - Copy all source files to output/evidence/working_copy/
  - Compute SHA-256 hash manifest of every file
  - Package immutable output/evidence/vault.zip and vault.zip.sha256
              │
              ▼
    [2. Extraction and Normalization]
  - Execute SQL queries (handles, messages, addresses)
  - Normalize telephone numbers to E.164 standard
  - Join messages, handles, and resolved contacts into JSON
              │
              ▼
    [3. Reporting and Filtering]
  - Apply optional date window or contact GUID scoping
  - Compute summary metrics (stats.json)
  - Render static HTML web pages using Maud templates
              │
              ▼
    [4. Delivery Packaging]
  - Optionally assemble iOS filesystem hierarchy (Cellebrite export)
  - Consolidate report, evidence vault, and verification scripts into deliverable.zip
  - Generate SHA-256 deliverable checksum
```

## The Role of Apple Messages Synchronization

Under modern macOS versions, Messages in iCloud operates as a bidirectional synchronizer. Rather than acting as a simple view into a cloud database, macOS maintains a full, active SQLite database locally at:

```text
~/Library/Messages/chat.db
```

When an Apple Account signs in to a Mac and enables Messages in iCloud, the sync engine downloads conversation histories and attachments onto local storage. If Text Message Forwarding is also enabled on the custodian's iPhone, standard SMS/MMS messages are relayed over Apple Push Notification services and written into the same local `chat.db`.

Because the tool reads this local database, no network calls or Apple API interactions occur during acquisition.

## The Chat Database and SQLite WAL Mode

`chat.db` operates in SQLite Write-Ahead Logging (WAL) mode. When active:

1. **`chat.db`**: Stores the base database pages.
2. **`chat.db-wal`**: Contains uncommitted transactions and recently written pages that have not yet been checkpointed back into `chat.db`.
3. **`chat.db-shm`**: A shared-memory index coordinating concurrent readers and writers accessing the WAL.

### Forensic Significance of the WAL

Forensic examiners must never acquire `chat.db` in isolation. Recent messages, read receipts, delivery statuses, or deletions often reside entirely inside `chat.db-wal`. 

If `chat.db` were copied alone without its companion WAL, all uncheckpointed transactions would be absent, producing an incomplete and potentially corrupt acquisition. `apple-message-forensics` discovers and preserves all three components together.

## Address Book Resolution

Within `chat.db`, message participants are tracked in the `handle` table solely by raw communication handles (e.g. `+15551234567` or `custodian@example.com`). Display names, nicknames, and organizational affiliations are not stored in the message database.

Instead, macOS resolves these handles at display time against the macOS AddressBook subsystem. Because users frequently configure multiple contact accounts (e.g., local contacts, iCloud contacts, Google Workspace, Exchange CardDAV), AddressBook data may be dispersed across several directories under:

```text
~/Library/Application Support/AddressBook/
```

To resolve participant names accurately:
1. The tool enumerates contact containers using a macOS system script (`scripts/list-containers.swift`) or fallback paths.
2. Contacts, phone numbers, and email mappings are extracted from `AddressBook-v22.abcddb`.
3. Phone numbers are normalized using `rlibphonenumber` to the E.164 standard so that divergent representations (such as `(555) 123-4567` and `+15551234567`) match reliably.
4. When an identifier cannot be matched to any contact record, the original handle is preserved as the contact identity.

## Separation of Evidence and Derived Artifacts

A core principle of forensic science is that acquisition must be non-destructive and verifiable. `apple-message-forensics` strictly separates:

- **Primary Evidence**: The exact byte-for-byte copies of `chat.db`, WAL sidecars, contact databases, and attachments stored in `evidence/vault.zip` and verified by SHA-256 manifests.
- **Derived Products**: JSON query dumps, summary statistics, and HTML reports. Derived files can be filtered, re-generated, or deleted without affecting the integrity of the underlying acquired evidence.
