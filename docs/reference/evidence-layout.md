# Evidence and Directory Layout Reference

This document specifies the filesystem layout, artifact packaging structure, and storage hierarchy produced during collection and delivery.

## Complete Case Folder Hierarchy

A completed run generates the following directory structure inside the configured output directory (default: `output/`):

```text
<output-dir>/
├── evidence/                         # Preserved raw evidence artifacts
│   ├── vault.zip                     # Compressed archive of original source files
│   ├── vault.zip.sha256              # Checksum manifest for vault.zip
│   └── working_copy/                 # Uncompressed working copy used during processing
│       ├── chat.db                   # Main SQLite Messages database
│       ├── chat.db-wal               # SQLite Write-Ahead Log (when present)
│       ├── chat.db-shm               # SQLite Shared Memory file (when present)
│       ├── AddressBook-v22.abcddb    # Contacts SQLite database
│       ├── Attachments/              # Copied message attachment directory
│       └── hash_manifest.txt         # SHA-256 hashes of all files in working_copy
│
├── json/                             # Structured query extractions
│   ├── handles.json                  # Participant identifiers (phone/email)
│   ├── messages.json                 # Parsed message records
│   ├── addresses.json                # Parsed contact records from AddressBook
│   ├── joined.json                   # Normalized messages joined with contact metadata
│   ├── filtered.json                 # Messages matching active date/contact filters
│   └── stats.json                    # Quantitative metrics computed across the dataset
│
├── report/                           # Standalone static HTML website
│   ├── index.html                    # Main report landing view
│   ├── contacts.html                 # Contact listing
│   ├── chats.html                    # Chat thread listing
│   ├── group_chats.html              # Multi-party chat listing
│   ├── messages/                     # Paged message listings (page_1.html, ...)
│   ├── media/                        # Attachment index and copied media assets
│   ├── assets/                       # Bundled CSS stylesheets (e.g. all.min.css)
│   ├── reader                        # Portable desktop report server (macOS/Linux)
│   └── reader.exe                    # Portable desktop report server (Windows)
│
├── cellebrite/                       # iOS-formatted filesystem (via cellebrite command)
│   ├── fake_iPhone.zip               # Archived iOS filesystem image
│   ├── fake_iPhone.zip.sha256        # Checksum manifest
│   └── private/var/mobile/Library/
│       ├── AddressBook/              # AddressBook SQLite databases
│       └── SMS/
│           ├── sms.db                # Renamed from chat.db
│           ├── sms.db-wal            # Renamed from chat.db-wal
│           └── Attachments/          # Message attachments
│
└── delivery/                         # Final examiner deliverable (via deliver command)
    ├── deliverable.zip               # Consolidated ZIP package for counsel
    ├── deliverable.zip.sha256        # SHA-256 checksum manifest
    ├── report.zip                    # Standalone archive of HTML report
    ├── vault.zip                     # Standalone archive of evidence vault
    └── validate_hashes.ps1           # PowerShell hash validation helper script
```

## Evidence Vault Components

The `evidence/` directory serves as the immutable acquisition record:

- **`working_copy/hash_manifest.txt`**: Records the SHA-256 hash and relative path of every acquired file immediately upon copy.
- **`vault.zip`**: Standard ZIP archive enclosing all original source files.
- **`vault.zip.sha256`**: Plaintext manifest recording the single SHA-256 checksum for `vault.zip`.

## Delivery Package

The `delivery/deliverable.zip` archive contains:

1. `report.zip`: Standalone report files with embedded readers.
2. `vault.zip`: Complete evidence package with original timestamps.
3. `fake_iPhone.zip` (if generated): Ready for direct import into Cellebrite Physical Analyzer.
4. `validate_hashes.ps1`: Script for Windows verification.
