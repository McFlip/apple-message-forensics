# How to Run an Evidence Collection

This guide shows how to collect Apple Messages artifacts, extract conversations to JSON, and generate HTML reports.

## Prerequisites

- Full Disk Access granted to your terminal application (see [How to Grant Full Disk Access](grant-full-disk-access.md)).
- The target Apple Account logged in on the collection Mac, with Messages sync and Text Message Forwarding enabled.
- Sufficient disk space for preserved attachments.

## Collect Everything and Generate Report (Default)

To run a complete acquisition and produce a report in a single step:

```bash
apple-message-forensics
```

By default, files are written to an `output/` subfolder in the current working directory.

### Specify an Output Directory

To store artifacts in a designated directory:

```bash
apple-message-forensics --output /path/to/case-output
```

> **Warning:** The command fails if the target output directory already exists and is not empty.

### Attach Case Metadata to the Report

Provide a YAML file containing case, examiner, and custodian information to include in the report header:

```bash
apple-message-forensics --meta /path/to/metadata.yaml
```

For format specifications, see [Case Metadata Schema in Reference](../reference/report-format.md#case-metadata-file).

### Change the Phone Number Region

Phone numbers are normalized to E.164 using country defaults. The default region is `US`. To change the country code:

```bash
apple-message-forensics --region DE
```

## Collection Only (Skip Report)

To preserve evidence and produce raw JSON extractions without compiling the HTML report:

```bash
apple-message-forensics collect --output /path/to/case-output
```

The evidence archive, SQLite database dumps, and query JSON files will be created under the output folder.

## Generate a Report from an Existing Collection

To generate a new HTML report from previously collected data (for example, after applying new filters or metadata):

```bash
apple-message-forensics --output /path/to/case-output report
```

Reports are placed in `output/report/` with timestamps. Successive report runs will not overwrite previously generated reports.

## Automated Workflow (EZ Mode)

If using `just` from the repository:

```bash
just do-it
```

This runs the end-to-end workflow to preserve evidence and export to Cellebrite format.
