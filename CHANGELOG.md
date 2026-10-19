# Changelog

## [Unreleased]

### Added

- initial Beta
- Archives source data to ZIP with SHA256 hash manifest
- HTML report with Windows reader.exe that launches bundled web server and launches default browser to `localhost`
- Cellebrite export that creates a ZIP file to parse in `Physical Analyzer` as a legacy file system collection

### Known Issues

- Address Book parsing is limited to Name, Phone Numbers, and Emails. Original files are still preserved in full.
- Don't run this from an exFAT external drive. Extra `._` "fork files" will be created that are not in the manifest and hash validation will fail.
- [ ] Need to add macOS Intel build

