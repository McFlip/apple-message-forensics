# Tutorial: Running Your First Apple Messages Collection

In this tutorial, you will perform a complete end-to-end collection using simulated forensic fixtures. You will inspect the preserved evidence, view the generated JSON extractions, and open the static HTML report in your browser.

By following these steps, you will learn the standard collection workflow without modifying real messages on your machine.

---

## What You Will Need

- A terminal on macOS or Linux.
- A cloned copy of this repository.
- A working Rust development environment, or pre-compiled `apple-message-forensics` binary.

---

## Step 1: Prepare the Working Directory

First, navigate to your repository root in your terminal:

```bash
cd apple-message-forensics
```

Confirm that the test fixture directory exists:

```bash
ls tests/fixtures/fakehome/Library/Messages
```

You should see `chat.db` and the `Attachments` folder listed.

---

## Step 2: Execute the Collection

We will now run `apple-message-forensics`, directing it to use the fixture home directory and sample case metadata:

```bash
cargo run -- \
  --home tests/fixtures/fakehome \
  --containers-json tests/fixtures/fake-containers.json \
  --meta tests/fixtures/mock_metadata.yaml \
  --output sample_output
```

During execution, the tool will:
1. Copy the source message database, attachments, and address book into `sample_output/evidence/working_copy/`.
2. Compute SHA-256 hashes of every acquired file.
3. Package an immutable evidence vault (`vault.zip`).
4. Execute SQL queries to parse handles, messages, and contacts into JSON.
5. Compile the static HTML report under `sample_output/report/`.

---

## Step 3: Inspect the Acquired Evidence

Check the contents of the generated `sample_output/evidence` directory:

```bash
ls -la sample_output/evidence/
```

You will see:
- `vault.zip`: An encrypted or compressed zip containing all source files as found.
- `vault.zip.sha256`: The SHA-256 checksum verifying the archive.
- `working_copy/`: The uncompressed files used during processing.

Now view the hash manifest generated during acquisition:

```bash
cat sample_output/evidence/working_copy/hash_manifest.txt
```

Notice each file has its cryptographic SHA-256 hash recorded beside its relative path.

---

## Step 4: Examine the Extracted JSON

Next, look inside the `sample_output/json` directory:

```bash
ls sample_output/json
```

Open `stats.json` to view summary metrics computed across the dataset:

```bash
cat sample_output/json/stats.json
```

You will see counts of messages, resolved contacts, and identified conversation threads.

---

## Step 5: View the Examination Report

Open the generated HTML report in your default web browser:

On macOS:
```bash
open sample_output/report/index.html
```

On Linux:
```bash
xdg-open sample_output/report/index.html
```

Observe the following elements in the opened dashboard:
- The case metadata header showing the sample examiner and case information.
- The summary statistics tiles.
- Navigation links to **Contacts**, **Chats**, **Group Chats**, and **Media Gallery**.

---

## Step 6: Clean Up

When you are finished reviewing the output, remove the generated sample directory:

```bash
rm -rf sample_output
```

---

## Summary

You have successfully:
- Acquired message and contact artifacts into an evidence vault.
- Inspected cryptographic SHA-256 manifests.
- Verified structured JSON output.
- Viewed the generated forensic HTML report.

To collect live data from an actual macOS user account, proceed to [How to Run an Evidence Collection](../how-to/run-a-collection.md).
