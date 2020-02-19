#!/usr/bin/env bash

set -euo pipefail

if [[ $* == *--rehash* ]] 
then
  cargo run --release --bin snapshot -- --rehash
else
  cargo test --workspace --release
  pwd
  cargo run --release --bin snapshot
fi
