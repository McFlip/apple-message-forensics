# Project TODO List

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

## ⚙️ CLI & Core
- [x] Output Directory Validation: Ensure the tool fails if the output directory is not empty (except for the `report` subcommand).
- [ ] Report Versioning: Implement the timestamped subfolder logic (`yyyy-mm-dd-HH-mm-ss`) for the `report` subcommand to prevent overwriting previous reports.

## 🧪 Verification
- [ ] End-to-End Testing: Verify the full flow: `collect` $\rightarrow$ `report` (with filters) $\rightarrow$ `deliver`.
- [x] Hash Validation: Ensure the generated hash files are compatible with `sha256sum -c`.
