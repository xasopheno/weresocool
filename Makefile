build_static:
	PORTAUDIO_ONLY_STATIC=true RUSTFLAGS='-L /usr/local/opt/portaudio/lib/' cargo build --release

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


test:
	cargo test --workspace --release
	cargo run --release --bin snapshot

test_rehash:
	cargo run --release --bin snapshot -- --rehash

scratch:
	cargo watch --exec "run --release --bin scratch"
