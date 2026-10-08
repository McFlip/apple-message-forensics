# Project TODO List

### ‼ PRIORITY

- [ ] Refactor Message Schema
  - [x] Investigate possible "to" field
    - if `is_from_me`
      - if `chat` is phone number or email - look up contact
      - if `chat` is a group chat id then `chat`
    - else custodian
  - [x] `"is_from_me"`
  - [x] populate contact with recipient for messages sent by custodian
    - done for DMs
    - must handle group chats differently
  - [ ] `"is_delivered"`
  - [ ] `"is_sent"`
  - [ ] `"is_read"`
  - [ ] `"date_edited"`
- [ ] Investigate other useful fields
  - [ ] Discuss with team
  - [ ] AI research

## 🛠️ Collection & Preservation
- [x] Evidence Packaging: Implement the `evidence` ZIP collection, including all source artifacts (chat.db, WAL, SHM, attachments, and AddressBook data).
- [x] Hashing: Implement MD5 and SHA256 hashing for all collected source files.
- [ ] Delivery Module: Implement the `deliver` subcommand to create a final ZIP archive of the case with optional password protection and a manifest of hashes.

## 📊 Report Enhancements
### Filtering Logic
- [x] Date Range Filtering: Implement `--start` and `--end` flags to filter messages in the generated HTML report.
- [ ] Contact Filtering: Implement `--filter-contacts` (comma-separated IDs) to limit the report to specific individuals.

### Report Content
- [ ] Header Statistics:
    - [ ] Add report generation timestamp (UTC and Local).
    - [ ] Add tool version.
    - [ ] Display active filter settings used for the current report.
    - [ ] Add basic stats: Total messages, earliest message date, and latest message date.
    - [ ] Implement "Top Talkers" summary.
- [ ] Message attachments
  - [ ] Copy attachments folder from working copy to report folder
  - [ ] Strip `~/Library/Messages/` from beginnig of the path
  - [ ] lowercase?
  - [ ] Re-encode `heic` media for web
  - [ ] vcf contact cards?
  - [ ] display inline with link to file
- [ ] Packaging
  - [ ] Embedded Python with PowerShell script to run `http.server`
  - [ ] Print to PDF
  - [ ] Export to CSV
- [ ] Group Chats
  - [ ] Create `group.json` - filter on `messages.json` looking for "chat[NUM]" regex in `.chat` field
  - [ ] Create group listing page and link in home.
  - [ ] List groups where custodian is a member inside the custodian detail page.
  - [ ] List DM thread seperately - DMs are just group chats with 2 members.
- [ ] Message card
  - [ ] Show `Sender -> Receiver`
  - [ ] Rename `chat` to `follow thread [chatName]` for DM and `go to group chat` for group chats
- [ ] Unkown contact page

## ⚙️ CLI & Core
- [x] Output Directory Validation: Ensure the tool fails if the output directory is not empty (except for the `report` subcommand).
- [ ] Report Versioning: Implement the timestamped subfolder logic (`yyyy-mm-dd-HH-mm-ss`) for the `report` subcommand to prevent overwriting previous reports.

## 🧪 Verification
- [ ] End-to-End Testing: Verify the full flow: `collect` $\rightarrow$ `report` (with filters) $\rightarrow$ `deliver`.
- [x] Hash Validation: Ensure the generated hash files are compatible with `sha256sum -c`.
