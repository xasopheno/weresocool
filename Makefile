package: 
	cd application && yarn package

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
	cargo clippy --all-targets -- -D warnings


test_application: 
	cd application && \
	yarn ts && \
	yarn lint && \
	yarn package && \
	yarn test \

test_rust:
	make clippy
	cargo test --workspace --release
	cargo run --release --bin snapshot

test_rehash:
	cargo run --release --bin snapshot -- --rehash

scratch:
	cargo watch --exec "run --release --bin scratch"

dev: 
	(cd application && yarn build-backend && yarn dev)
