#!/usr/bin/env sh

if cargo run --release working.socool; then
    printf 'Parser Test Passed\n'
else
    printf '!!!!!Parser Test Failed!!!!!\n'
fi
