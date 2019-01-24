#!/usr/bin/env sh

./format.sh
echo "Formatted"
cargo test
(cd parser && cargo test)
(cd socool_ast && cargo test)
cargo run --bin snapshot
