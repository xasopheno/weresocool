format:
	cargo fmt --all

clippy:
	# cargo +nightly clippy --all-targets -- -D warnings
	cargo clippy --all-targets -- -D warnings

test:
	cargo test --workspace --release
	cargo run --release --bin snapshot

test_rust_generated:
	cargo test --release _generated

test_rehash:
	cargo run --release --bin snapshot -- --rehash

check-licenses: 
  cargo deny check licenses --hide-inclusion-graph

