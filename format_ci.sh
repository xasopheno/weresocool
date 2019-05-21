#!/usr/bin/env bash

set -euo pipefail

cargo fmt
pushd parser && cargo fmt -- --check && popd
pushd ast && cargo fmt -- --check && popd
