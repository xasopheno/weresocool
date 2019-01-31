#!/usr/bin/env bash

set -euo pipefail

./format.sh
echo "Formatted"
pushd socool_ast; cargo test; popd
pushd parser; cargo test; popd
pushd parser; ./parser_test.sh; popd
cargo test
pwd
cargo run --bin snapshot
