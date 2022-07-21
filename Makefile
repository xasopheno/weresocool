test:
	make test_rust && make test_application

format:
	#!/usr/bin/env bash
	set -euo pipefail

	cargo fmt
	pushd parser && cargo fmt && popd
	pushd ast && cargo fmt && popd

format_ci:
	#!/usr/bin/env bash
	set -euo pipefail

	cargo fmt
	pushd parser && cargo fmt -- --check && popd
	pushd ast && cargo fmt -- --check && popd

clippy:
	# cargo +nightly clippy --all-targets -- -D warnings
	cargo clippy --all-targets -- -D warnings

test_rust:
	cargo test --workspace --release
	cargo run --release --bin snapshot

test_rust_generated:
	cargo test --release _generated

test_rehash:
	cargo run --release --bin snapshot -- --rehash
