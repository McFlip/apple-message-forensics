# How to Filter Reports

This guide explains how to filter report output by date range, specific contact identifiers, or a combination of both.

Filtering does not alter collected evidence. All source files and raw database dumps are preserved unconditionally in the evidence vault. Filters only scope the messages included in the generated HTML report and `stats.json`.

## Filter by Date Range

Filter messages by RFC 3339 timestamps using `--start`, `--end`, or both. Dates are inclusive.

### Filter from a Date Forward

To include messages sent or received on or after a specific date:

```bash
apple-message-forensics --start 2026-01-01T00:00:00Z
```

### Filter up to an End Date

To include messages up to a specific date:

```bash
apple-message-forensics --end 2026-03-31T23:59:59Z
```

### Filter Between Dates

To scope a specific investigation window:

```bash
apple-message-forensics --start 2026-01-01T00:00:00Z --end 2026-03-31T23:59:59Z
```

## Filter by Contact IDs

To restrict the report to messages involving specific contacts:

1. Perform a collection (or run `apple-message-forensics collect`) to generate `json/addresses.json`.
2. Inspect `json/addresses.json` to identify the `contact_unique_id` GUIDs for the custodians or targets of interest.
3. Create a plain-text file (for example, `contact_ids.txt`) containing one `contact_unique_id` per line:

   ```text
   123E4567-E89B-12D3-A456-426614174000
   987F6543-E21A-34C5-B789-123456789ABC
   ```

4. Pass the file using `--contact-ids-file`:

   ```bash
   apple-message-forensics --contact-ids-file contact_ids.txt
   ```

## Combine Date and Contact Filters

You can pass both filters together. The contact-ID filter is applied first, followed by the date range filter:

```bash
apple-message-forensics \
  --contact-ids-file contact_ids.txt \
  --start 2026-01-01T00:00:00Z \
  --end 2026-03-31T23:59:59Z
```

## Re-Filtering an Existing Collection

To re-filter existing evidence without repeating acquisition:

```bash
apple-message-forensics \
  --output /path/to/case-output \
  --contact-ids-file contact_ids.txt \
  --start 2026-01-01T00:00:00Z \
  report
```

Any previous `filtered.json` is automatically cleared before applying the new filters.
