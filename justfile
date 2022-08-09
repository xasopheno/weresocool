build:
	cargo build
build-release:
	cargo build -- release

format:
	cargo fmt --all

clippy:
	# cargo +nightly clippy --all-targets -- -D warnings
	cargo clippy --all-targets -- -D warnings

test:
	cargo nextest run --workspace --release
	just test_snapshot

test_generated:
	cargo nextest run --release _generated

test_snapshot:
	cargo run --release --bin snapshot
test_rehash:
	cargo run --release --bin snapshot -- --rehash

check-licenses: 
  cargo deny check licenses --hide-inclusion-graph

test-github-actions:
	act
