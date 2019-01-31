#!/usr/bin/env bash

set -euo pipefail

printf 'Running Parser Tests\n'

cargo run --release working.socool
