#!/usr/bin/env bash

set -euo pipefail

cargo fmt
pushd parser && cargo fmt && popd
pushd ast && cargo fmt && popd
