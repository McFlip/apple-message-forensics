# How to Deliver Evidence and Use the Reader

This guide covers packaging evidence and reports into deliverables for counsel, exporting evidence into Cellebrite-compatible formats, and running the bundled offline report reader.

## Create a Cellebrite-Style Filesystem

To convert collected macOS evidence into an iOS-style filesystem structure that can be ingested into Cellebrite Physical Analyzer:

```bash
apple-message-forensics --output /path/to/case-output cellebrite
```

The command:
- Verifies all source files against `evidence/working_copy/hash_manifest.txt`.
- Mirrors the AddressBook and Messages directories to `cellebrite/private/var/mobile/Library/`.
- Renames `chat.db` and any SQLite sidecars (`chat.db-wal`, `chat.db-shm`) to `sms.db` within the `SMS` directory.
- Creates `cellebrite/fake_iPhone.zip` along with its SHA-256 checksum manifest.

## Package the Deliverable Bundle

The `deliver` subcommand gathers `report.zip`, `vault.zip`, `validate_hashes.ps1`, and Cellebrite exports into a consolidated package.

Run:

```bash
apple-message-forensics --output /path/to/case-output deliver
```

This creates:
- `delivery/deliverable.zip`
- `delivery/deliverable.zip.sha256`

### Password-Protect the Deliverable ZIP

To encrypt `deliverable.zip` using AES encryption, pass the `--protect-deliverable` flag:

```bash
apple-message-forensics --output /path/to/case-output --protect-deliverable deliver
```

Set the password via the `DEL_PW` environment variable:

```bash
export DEL_PW="YourStrongPassphrase"
apple-message-forensics --output /path/to/case-output --protect-deliverable deliver
```

If `DEL_PW` is unset, you are prompted interactively in the terminal. Passwords cannot be passed as raw command-line flags.

## Use the Offline Report Reader

The HTML report includes dynamic views and thread navigation that require an HTTP server. To support non-technical reviewers such as legal counsel, a portable desktop reader is bundled with reports.

### Start the Reader

1. In the delivered report package or `output/report/` folder, run the reader executable:

   - **macOS / Linux**: `./reader`
   - **Windows**: `reader.exe`

2. The reader automatically spawns the local Axum web server and opens the report in the system default web browser.

### Exit the Reader

To shut down the background server cleanly, click the **Exit** button in the reader window or press `Ctrl+C` in the terminal session running it.
