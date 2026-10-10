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

