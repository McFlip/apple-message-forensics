# Project To Do List

## Refactor Message Schema
  - [x] Investigate possible "to" field
    - if `is_from_me`
      - if `chat` is phone number or email - look up contact
      - if `chat` is a group chat id then `chat`
    - else custodian
  - [x] `"is_from_me"`
  - [x] populate contact with recipient for messages sent by custodian
    - done for DMs
    - must handle group chats differently
  - [x] `"is_delivered"`
  - [x] `"is_sent"`
  - [x] `"is_read"`
  - [x] `"date_edited"`
- [ ] Investigate other useful fields
  - [ ] Discuss with team
  - [ ] Discuss with `Nuix` support
  - [ ] AI research

## 🛠️ Collection & Preservation
- [x] Evidence Packaging: Implement the `evidence` ZIP collection, including all source artifacts (chat.db, WAL, SHM, attachments, and AddressBook data).
- [x] Hashing: Implement MD5 and SHA256 hashing for all collected source files.
- [ ] Delivery Module: Implement the `deliver` subcommand to create a final ZIP archive of the case with optional password protection and a manifest of hashes.
  - [x] ZIP creation
  - [ ] Password protection

## 📊 Report Enhancements
### Filtering Logic
- [x] Date Range Filtering: Implement `--start` and `--end` flags to filter messages in the generated HTML report.
- [x] Contact Filtering: Implement `--contact-ids-file` (plain‑text GUID list) to limit the report to specific individuals.

### Report Content
- [x] Header Statistics:
    - [x] Add report generation timestamp (UTC and Local).
    - [x] Add tool version.
    - [x] Display active filter settings used for the current report.
    - [x] Add basic stats: Total messages, earliest message date, and latest message date.
    - [x] Implement "Top Talkers" summary.
- [ ] Message attachments
  - [x] Copy attachments folder from working copy to report folder
  - [x] Strip `~/Library/Messages/Attachments/` from beginning of the path
  - [x] URL encode
  - [ ] Re-encode `heic` media for web - run this by the team
  - [x] vcf contact cards?
  - [x] Regression => Chat threads not showing `View attachment`
  - [ ] display inline with link to file? - run this by the team
    - NOTE: Added media page for easy browsing of media - will probably go with current behavior of showing link in message to raw content
- [ ] Packaging
  - [x] Embedded GUI app that launches server and then default browser to localhost
    - [x] Executables
    - [x] Update deliver command to copy binaries
    - [x] Vendor font dependency instead of using a CDN
  - [ ] Export to load file for `Nuix` - JSONL?
  - [x] Cellebrite
    - [x] Scaffold mock iOS file system
    - [x] Unzip Evidence to correct location
    - [x] Validate hashes
      - NOTE: can't hash the AddressBook due to data migration requirements
    - [x] Rename files to iOS version
      - [x] SMS.db
      - [x] AddressBook.sqlitedb
    - [x] Export `AddressBook-v22.abcddb` -> JSON -> `AddressBook.sqlitedb`
      - NOTE: requires data mapping and time conversions
    - [x] ZIP mock iOS file system to Delivery folder
    - [x] Hash delivery
- [x] Group Chats
  - [x] Create `group.json` - filter on `messages.json` looking for "chat[NUM]" regex in `.chat` field
  - [x] Create group listing page and link in home.
- [x] Message card
  - [x] Show `Sender -> Receiver`
- [x] Media page lists media attachments

## ⚙️ CLI & Core
- [x] Output Directory Validation: Ensure the tool fails if the output directory is not empty (except for the `report` subcommand).

## 🧪 Verification
- [x] Hash Validation: Ensure the generated hash files are compatible with `sha256sum -c`.
- [x] Validate timestamps against real data
- [x] Validate date filters against real data
- [x] Validate contact filter against real data
- [x] End-to-End Testing: Verify the full flow: `collect` $\rightarrow$ `report` (with filters) $\rightarrow$ `deliver`.
  - [x] Added `just` commands for E2E on a real system.
  - [x] Automated E2E for collect command using SQL test fixtures and temp file system

## Release
- [ ] Test on real targets
- [ ] Test on other examiner's machines
- [ ] Add macOS Intel build
