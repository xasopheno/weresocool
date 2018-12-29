#!/usr/bin/env sh

cargo fmt && cargo test
(cd parser && cargo fmt && cargo test)
