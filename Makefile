.PHONY: clippy dev format lint package run scratch test ts_test test_application test_rust test_rehash

ifeq ($(OSTYPE),darwin)
PACKAGE := application/release/mac/WereSoCool.app
else
PACKAGE := application/release/linux-unpacked/weresocool
endif

$(PACKAGE): application/node_modules
	cd application && yarn package

package: $(PACKAGE)

run: $(PACKAGE)
ifeq ($(OSTYPE),darwin)
	open $(PACKAGE)
else
	$(PACKAGE)
endif

dev: application/node_modules
	(cd application && yarn build-backend && yarn dev)

test:
	make test_rust && make test_application

lint:
	make clippy && \
	make ts_test 

ts_test: application/node_modules
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

test_application: application/node_modules
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

application/node_modules: application/yarn.lock
	cd application && yarn install
