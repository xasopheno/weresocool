package: 
	cd application && yarn package

run:
	./application/release/linux-unpacked/weresocool

dev: 
	(cd application && yarn build-backend && yarn dev)

test:
	make test_rust && make test_application

lint:
	make clippy && \
	make ts_test 

ts_test:
	#!/usr/bin/env bash
	set -euo pipefail

	cd application && \
	yarn ts && \
	yarn lint

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
	yarn package && \
	yarn test

test_rust:
	cargo test --workspace --release
	cargo run --release --bin snapshot

test_rehash:
	cargo run --release --bin snapshot -- --rehash

scratch:
	cargo watch --exec "run --release --bin scratch"

