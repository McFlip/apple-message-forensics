# Command-Line Options Reference

Reference specification of all command-line arguments, options, flags, and subcommands for `apple-message-forensics`.

## Synopsis

```text
apple-message-forensics [OPTIONS] [COMMAND]
```

## Global Options

The following options apply to the default command and all subcommands:

| Option | Shorthand | Type | Default | Description |
|---|---|---|---|---|
| `--output <DIR>` | `-o` | Path | `output` | Path to destination directory for collected data, databases, and reports. Must be non-existent or empty for new collections. |
| `--meta <FILE>` | `-m` | Path | None | Path to case metadata YAML file included in report headers. |
| `--start <DATE>` | None | RFC 3339 | None | Lower timestamp boundary (inclusive) for message reporting filter (e.g. `2026-01-01T00:00:00Z`). |
| `--end <DATE>` | None | RFC 3339 | None | Upper timestamp boundary (inclusive) for message reporting filter (e.g. `2026-12-31T23:59:59Z`). |
| `--region <CODE>` | None | String | `US` | Two-letter ISO country code used for phone number normalization (e.g. `US`, `GB`, `DE`). |
| `--contact-ids-file <FILE>` | None | Path | None | Plain-text file containing `contact_unique_id` GUIDs (one per line) to scope reported messages. |
| `--home <DIR>` | None | Path | `$HOME` | Home directory scanned for evidence artifacts. Used primarily for testing fixtures. |
| `--protect-deliverable` | None | Flag | `false` | Enables AES password encryption for `delivery/deliverable.zip`. Uses `DEL_PW` environment variable or prompts interactively. |
| `--containers-json <FILE>` | None | Path | None | Path to pre-enumerated contact containers JSON. Overrides dynamic Swift discovery script. |
| `-h`, `--help` | None | Flag | None | Prints help information. |
| `-V`, `--version` | None | Flag | None | Prints version information. |

## Subcommands

| Subcommand | Description |
|---|---|
| *(none)* | Default execution. Performs full acquisition, extracts JSON, writes stats, and generates the HTML report. |
| `collect` | Runs acquisition and extraction to JSON. Skips HTML report generation. |
| `report` | Generates a timestamped HTML report from an existing `json/joined.json` dataset. |
| `cellebrite` | Constructs an iOS-structured directory layout (`fake_iPhone`) under `output/cellebrite/` suitable for Cellebrite Physical Analyzer ingestion. |
| `deliver` | Bundles `report.zip`, `vault.zip`, Cellebrite exports, and validation scripts into `delivery/deliverable.zip`. |
| `find-icloud-contacts` | Scans macOS AddressBook container records to discover and print the active iCloud synchronized contact container path. |

## Environment Variables

| Variable | Description |
|---|---|
| `DEL_PW` | Password used by `--protect-deliverable` when packaging `delivery/deliverable.zip`. If unset and the flag is passed, the CLI prompts on stdin. |
| `HOME` | Standard macOS user home directory. Used when `--home` is omitted. |
| `RUST_LOG` | Configures logging output level (`error`, `warn`, `info`, `debug`, `trace`). |
