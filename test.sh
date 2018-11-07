#!/usr/bin/env bash

# Main package
cargo test

# Rjs package (for wasm)
cd rjs-parse
cargo test
cd -

# from-file package
cd from-file
cargo test
