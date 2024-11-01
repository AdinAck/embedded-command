#!/bin/bash

set -euxo pipefail

rustup toolchain install nightly --component miri

TARGETS=("thumbv6m-none-eabi" "thumbv7em-none-eabi" "thumbv7em-none-eabihf")
CRATES=("macros" "cookie-cutter" "dispatch-bundle")

# build

for TARGET in "${TARGETS[@]}"; do
    rustup target add "$TARGET"
    for CRATE in "${CRATES[@]}"; do
        cargo build -p "$CRATE" --target "$TARGET"
    done
done

# tests

for CRATE in "${CRATES[@]}"; do
    cargo test -p "$CRATE"
done

# miri

cargo +nightly miri test -p embedded-command command_buffer

# clippy
cargo clippy

# crate-specific

# cookie-cutter

# asm analysis
cargo build -p cookie-cutter --bin asm --target thumbv7em-none-eabihf --features binary --release
