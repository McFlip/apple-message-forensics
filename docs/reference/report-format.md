# Report Format Reference

Technical specification of the static HTML examination report and its metadata schemas.

## Structure Overview

The report is compiled as a self-contained local website residing inside `output/report/`. It includes HTML templates styled with bundled assets and does not load external network CDNs.

### Report Page Inventory

| View | Path | Description |
|---|---|---|
| Index / Landing | `index.html` | Overview dashboard with case header, summary metrics, and navigation cards. |
| Contacts | `contacts.html` | Alphabetical directory of all resolved and unresolved contacts with handle associations. |
| Contact Thread | `contacts/<contact_id>.html` | Messages exchanged with an individual contact across all relevant chats. |
| Chat Threads | `chats.html` | Chronological list of one-on-one and group threads ordered by most recent activity. |
| Group Chats | `group_chats.html` | Listing of multi-party conversations and group participant membership. |
| All Messages | `messages/page_<n>.html` | Paged chronological ledger of all messages in the dataset. |
| Media Gallery | `media/index.html` | Catalog of recovered attachments (images, audio, videos, documents). |

## Header Elements

The report header displays:

- **Run Metadata**: Execution timestamp in UTC, `apple-message-forensics` software version.
- **Applied Filters**: Active start date, end date, contact filters, and normalization region.
- **Case Information**: Rendered from the optional metadata YAML file.
- **Dataset Metrics**:
  - Total messages acquired.
  - Total attachments recovered.
  - Top talkers / participant interaction volumes.

## Case Metadata File

Examiner and case identifiers can be injected into the report header via `--meta <path>`.

### YAML Specification

```yaml
case:
  name: "Investigation Name"
  number: "CASE-YYYY-NNNN"
  requestor: "Counsel or Requesting Agency"
  notes: |
    Narrative notes regarding acquisition scope, search warrant,
    or protective order limitations.

analyst:
  name: "Examiner Name"
  organization: "DFIR Laboratory"
  contact: "examiner@example.com"

custodian:
  name: "Account Custodian"
  device_name: "Custodian's Mac or iPhone model"

report:
  title: "Formal Report Title"
  date: "YYYY-MM-DD"
```

### Schema Rules

- All top-level blocks (`case`, `analyst`, `custodian`, `report`) are optional.
- Missing values render as empty cells in the HTML metadata card.
- Text in `notes` preserves line breaks.
