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
	cargo clippy --all-targets -- -D warnings

test:
	cargo test --workspace --release
	cargo run --release --bin snapshot

test_rehash:
	cargo run --release --bin snapshot -- --rehash

scratch:
	cargo watch --exec "run --release --bin scratch"

