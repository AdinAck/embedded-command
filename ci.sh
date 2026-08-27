#!/bin/bash

set -euxo pipefail

rustup toolchain install nightly --component miri

TARGETS=("thumbv6m-none-eabi" "thumbv7em-none-eabi" "thumbv7em-none-eabihf")
CRATES=("embedded-command-macros" "serac" "dispatch-bundle")

# build

for TARGET in "${TARGETS[@]}"; do
    rustup target add "$TARGET"
    for CRATE in "${CRATES[@]}"; do
        cargo build -p "$CRATE" --all-features --target "$TARGET"
    done
done

# tests

for CRATE in "${CRATES[@]}"; do
    cargo test -p "$CRATE" --all-features
done

# miri

# skip for now
# cargo +nightly miri test -p embedded-command command_buffer

# clippy
cargo clippy -- --deny warnings

# crate-specific

# serac
## asm analysis
cargo build -p serac --bin asm --target thumbv7em-none-eabihf --features binary --release
