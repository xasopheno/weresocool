format:
	#!/usr/bin/env bash
	set -euo pipefail

	cargo fmt --all

format_ci:
	#!/usr/bin/env bash
	set -euo pipefail

	cargo fmt --all --check

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
