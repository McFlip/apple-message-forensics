# Project To Do List

## Refactor Message Schema
- [ ] Investigate other useful fields
  - [ ] Discuss with team
  - [ ] Discuss with `Nuix` support
  - [ ] AI research

### Report Content
- [ ] Message attachments
  - [x] Copy attachments folder from working copy to report folder
  - [x] Strip `~/Library/Messages/Attachments/` from beginning of the path
  - [x] URL encode
  - [ ] Re-encode `heic` media for web - run this by the team
  - [x] vcf contact cards?
  - [x] Regression => Chat threads not showing `View attachment`
  - [ ] display inline with link to file? - run this by the team
    - NOTE: Added media page for easy browsing of media - will probably go with current behavior of showing link in message to raw content
- [ ] Packaging
  - [ ] Embedded Python to work around blocked binaries
  - [ ] Export to load file for `Nuix` - JSONL?

## Release
- [x] Test on real targets
- [ ] Test on other examiner's machines
- [x] `fakehome` test fixture broken due to using a real UUID collected by the Swift script query to Contacts
- [x] Password-protect deliverable ZIP via `--protect-deliverable` (DEL_PW env or prompt)
- [x] Add macOS Intel build
