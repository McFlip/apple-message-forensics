# apple-message-forensics

[![Documentation](https://img.shields.io/badge/docs-Diátaxis-blue.svg)](docs/index.md)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)

**Apple Messages Forensics** is an open-source DFIR and eDiscovery tool for collecting and preserving Apple Messages data from macOS, extracting messages and attachments to structured JSON, and generating human-readable HTML reports for counsel and investigators.

Apple Messages Forensics was inspired by the Python project [imessagedb](https://pypi.org/project/imessagedb/).
This project is written in Rust and has a few key differences.

- **Forensically Sound**: Preserves raw SQLite databases (`chat.db`, WAL sidecars) and attachments in an immutable vault with cryptographic SHA-256 manifests.
- **Whole-Dataset Extraction**: Dumps complete message histories joined with resolved contacts from macOS AddressBook.
- **Scoped Reporting**: Filter message reports by date range or specific contact IDs without touching preserved evidence.
- **Examiner Deliverables**: Generates static HTML reports, optional offline Windows/macOS desktop readers, and Cellebrite-compatible filesystem archives (`fake_iPhone.zip`).

---

## Quick Start

### Prerequisites

Install Homebrew, just, and 7zip

```bash
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/bin/install.sh)"
brew install just
brew install sevenzip
```


### 1. Install

Download the latest binary release for your macOS architecture from [Releases](https://github.com/McFlip/apple-message-forensics/releases)

Validate hash

```bash
sha256sum release.7z
```

Extract with 7zip

```bash
7zz x release.7z
```

Or install via Cargo:

```bash
cargo install --path .
```

### 2. Grant Full Disk Access

Terminal access to `~/Library/Messages` requires macOS Full Disk Access:

1. Open **System Settings** → **Privacy & Security** → **Full Disk Access**.
2. Enable access for your terminal application (Terminal, iTerm2, etc.). I use `Ghostty`, BTW.

For detailed steps with screenshots, see [How to Grant Full Disk Access](docs/how-to/grant-full-disk-access.md).

### 3. Run

Run an end-to-end acquisition, report generation, and package for delivery:

```bash
just do-it
```

The reader app will automatically launch your default browser and open the HTML report.

Results are in `output/delivery/deliverable.zip`

```bash
deliverable.zip.sha256  # sha256 hash for validating deliverable.zip
fake_iPhone.zip         # evidence file to process in Cellebrite Physical Analyzer
fake_iPhone.zip.sha256  # sha256 hash for validating fake_iPhone.zip
report.zip              # HTML report
validate_hashes.ps1     # PowerShell script to validate hashes in Windows
vault.zip               # Preserved artifacts
```

---

## Documentation

Comprehensive documentation is structured using the [Diátaxis](https://diataxis.fr) framework in the [`docs/`](docs/index.md) directory (hosted via GitHub Pages):

- **[Tutorials](docs/tutorials/first-collection.md)**
  - [Running Your First Apple Messages Collection](docs/tutorials/first-collection.md)
- **[How-To Guides](docs/how-to/run-a-collection.md)**
  - [Grant Full Disk Access](docs/how-to/grant-full-disk-access.md)
  - [Install apple-message-forensics](docs/how-to/install.md)
  - [Run an Evidence Collection](docs/how-to/run-a-collection.md)
  - [Filter Reports by Date or Contact ID](docs/how-to/filter-reports.md)
  - [Deliver Evidence, Cellebrite Export & Reader](docs/how-to/deliver.md)
  - [Build from Source](docs/how-to/build-from-source.md)
  - [Validate Evidence Hashes](docs/how-to/validate-hashes.md)
- **[Reference](docs/reference/cli-options.md)**
  - [Command-Line Options & Flags](docs/reference/cli-options.md)
  - [Report Format & Metadata YAML Schema](docs/reference/report-format.md)
  - [Evidence & Directory Layout](docs/reference/evidence-layout.md)
  - [Database Schemas & SQL Queries](docs/reference/schema-and-queries.md)
- **[Explanation](docs/explanation/how-it-works.md)**
  - [Scope, Purpose, and Technical Limitations](docs/explanation/scope-and-limitations.md)
  - [How it Works: Architecture & Evidence Lineage](docs/explanation/how-it-works.md)

---

## Issues & Contributing

If you encounter issues, bugs, or have feature suggestions, please open an issue on GitHub:
https://github.com/McFlip/apple-message-forensics/issues
