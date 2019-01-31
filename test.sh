#!/usr/bin/env sh

./format.sh
echo "Formatted"
(cd socool_ast && cargo test)
(cd parser && cargo test)
(cd parser && ./parser_test.sh)
cargo test
cargo run --bin snapshot
