# How to Validate Evidence Hashes

This guide provides instructions for independently verifying the cryptographic integrity of acquired evidence and deliverable packages using SHA-256 hashes.

## Prerequisites

- Access to the generated manifest files (`hash_manifest.txt`, `SHA256SUMS`, or `*.sha256`).
- Terminal access on macOS/Linux or PowerShell on Windows.

## Verify on macOS and Linux

Use standard coreutils `sha256sum` (or `shasum -a 256` on native macOS).

Navigate to the directory containing the manifest file:

```bash
cd /path/to/case-output/evidence/working_copy
sha256sum -c hash_manifest.txt
```

Or on native macOS without GNU coreutils:

```bash
shasum -a 256 -c hash_manifest.txt
```

### Expected Output

Each file line is evaluated and marked `OK`:

```text
chat.db: OK
chat.db-wal: OK
AddressBook-v22.abcddb: OK
```

If any file has been modified or corrupted, the command returns `FAILED` for that file and exits with a non-zero exit code.

## Verify on Windows with PowerShell

The repository and delivery bundles include a PowerShell validation script located at `scripts/validate_hashes.ps1`.

1. Open PowerShell.
2. Run the script and pass the path to the manifest file or its parent directory:

```powershell
.\scripts\validate_hashes.ps1 .\delivery\deliverable.zip.sha256
```

The script reads the expected hashes, computes the SHA-256 hash of each referenced file using `Get-FileHash`, and verifies matches.
