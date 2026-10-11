# Build lint and test
check:
  cargo check
  cargo clippy
  cargo test

# Clean output folder
clean:
  rm -rf output

# run but don't launch server
build: check clean
  cargo run -- --meta tests/fixtures/mock_metadata.yaml

# Full run
full: build
  just serve

# Just rerun the report
report: check
  rm -rf output/report
  cargo run -- --meta tests/fixtures/mock_metadata.yaml report
  just serve

# Launch server for report
serve:
  python3 -m http.server --directory output/report/

# E2E test filtering on contact id
filter_contact: check clean
  cargo run -- --meta tests/fixtures/mock_metadata.yaml --contact-ids-file tests/fixtures/mock_contact_id
  just serve

# E2E test filtering on start date
filter_start: check clean
  cargo run -- --meta tests/fixtures/mock_metadata.yaml --start 2025-01-01T00:00:00Z
  just serve

# E2E test filtering on both start date and contact id
filter_both: check clean
  cargo run -- --meta tests/fixtures/mock_metadata.yaml --start 2025-01-01T00:00:00Z --contact-ids-file tests/fixtures/mock_contact_id
  just serve

# Deliver
deliver: build
  cargo run -- deliver
