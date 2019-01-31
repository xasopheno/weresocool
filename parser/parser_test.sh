#!/usr/bin/env sh

printf 'Running Parser Tests\n'

if cargo run --release working.socool; then
    printf 'Parser Test Passed\n'
else
    printf '!!!!!Parser Test Failed!!!!!\n'
fi
