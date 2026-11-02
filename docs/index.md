# Apple Messages Forensics Documentation

Welcome to the documentation for **Apple Messages Forensics**, an open-source DFIR and eDiscovery tool for collecting and preserving Apple Messages data from macOS, extracting messages and attachments to structured JSON, and generating static HTML reports for counsel and investigators.

This documentation is organized using the **Diátaxis** documentation framework into four distinct sections:

---

## [Tutorials](tutorials/first-collection.md)
*Learning-oriented lessons for newcomers.*

- **[Running Your First Apple Messages Collection](tutorials/first-collection.md)**: A step-by-step hands-on walkthrough using mock test fixtures to learn the tool without touching real data.

---

## [How-To Guides](how-to/run-a-collection.md)
*Goal-oriented directions for accomplishing specific real-world tasks.*

- **[Grant Full Disk Access](how-to/grant-full-disk-access.md)**: Enable terminal access to protected macOS Messages and AddressBook folders.
- **[Install apple-message-forensics](how-to/install.md)**: Download release binaries and add them to your system path.
- **[Run an Evidence Collection](how-to/run-a-collection.md)**: Execute single-command collections, customize output paths, and attach metadata.
- **[Filter Reports](how-to/filter-reports.md)**: Scope reporting by date windows or specific contact IDs without modifying evidence.
- **[Deliver Evidence and Use Reader](how-to/deliver.md)**: Package deliverable ZIPs, generate Cellebrite-compatible formats, and use the desktop report viewer.
- **[Build from Source](how-to/build-from-source.md)**: Compile native binaries and cross-compile the Windows reader.
- **[Validate Evidence Hashes](how-to/validate-hashes.md)**: Verify SHA-256 manifests on macOS, Linux, or Windows.

---

## [Reference](reference/cli-options.md)
*Information-oriented technical descriptions of machinery, schemas, and formats.*

- **[Command-Line Options](reference/cli-options.md)**: Full synopsis of flags, subcommands, and environment variables.
- **[Report Format](reference/report-format.md)**: Static HTML report layout and YAML case metadata schema.
- **[Evidence and Directory Layout](reference/evidence-layout.md)**: Detailed folder layout of preserved vaults and deliverables.
- **[Database Schema and SQL Queries](reference/schema-and-queries.md)**: Technical breakdown of macOS `chat.db`, `AddressBook`, and extraction SQL.

---

## [Explanation](explanation/how-it-works.md)
*Understanding-oriented discussions of architecture, context, and forensic lineage.*

- **[Scope, Purpose, and Technical Limitations](explanation/scope-and-limitations.md)**: Architectural purpose, comparison with mobile physical extraction tools, and limitations.
- **[How it Works: Architecture and Evidence Lineage](explanation/how-it-works.md)**: Synchronization mechanisms, SQLite WAL forensics, contact resolution, and data lineage.
