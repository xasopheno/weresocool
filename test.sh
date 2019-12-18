#!/usr/bin/env bash

set -euo pipefail

if [[ $* == *--rehash* ]] 
then
  cargo run --release --bin snapshot -- --rehash
else
  pushd ast; cargo test; popd
  pushd parser; cargo test; popd
  pushd parser; ./parser_test.sh; popd
  cargo test --release
  pwd
  cargo run --release --bin snapshot
fi
