#!/usr/bin/env bash

set -euo pipefail

cargo fmt
(cd parser && cargo fmt)
(cd socool_ast && cargo fmt)
