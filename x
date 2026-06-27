#!/bin/bash -e

# `bootstrap` is excluded from the workspace default-members (it is native-only and
# does not build for wasm), so select its package explicitly here.
cargo build -p bootstrap --bin bootstrap --release

./target/release/bootstrap "$@"
