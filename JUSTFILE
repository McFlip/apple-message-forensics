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
  cargo run -- --meta tests/fixtures/mock_metadata.yaml --home tests/fixtures/fakehome

# build but with real data
build-real: check clean
  cargo run -- --meta tests/fixtures/mock_metadata.yaml

# Build the report reader and its server
build-reader:
  cargo build --release --package report-reader --bins
  cp target/release/reader target/release/report-server bin/

# Bundle the report reader and its server with the report
bundle-reader: build-reader
  test -f output/report/index.html
  cp bin/reader bin/report-server output/report/

# Full run
full: build
  just serve

# Full run but with real data
real-deal: build-real
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
  cargo run -- --meta tests/fixtures/mock_metadata.yaml --contact-ids-file tests/fixtures/mock_contact_id --home tests/fixtures/fakehome
  just serve

# E2E test filtering on start date
filter_start: check clean
  cargo run -- --meta tests/fixtures/mock_metadata.yaml --start 2024-06-31T14:00:00Z --home tests/fixtures/fakehome
  just serve

# E2E test filtering on both start date and contact id
filter_both: check clean
  cargo run -- --meta tests/fixtures/mock_metadata.yaml --start 2024-06-02T03:00:00Z --contact-ids-file tests/fixtures/mock_contact_id --home tests/fixtures/fakehome
  just serve

# Deliver
deliver: build build-reader
  cargo run -- deliver

# Release build
release: check
  rm -rf release
  mkdir -p release/bin
  cargo build --release --bins
  cp target/release/reader target/release/report-server release/bin/
  cp target/release/apple-message-forensics release/
  cp just/JUSTFILE release/
  cp README.md release/
  cp LICENSE release/
  cp screenshot{1,2}.png release/
  cp tests/fixtures/mock_contact_id release/contact_id.txt
  cp tests/fixtures/mock_metadata.yaml release/metadata.yaml
  mkdir release/scripts
  cp -r scripts/ release/scripts/

