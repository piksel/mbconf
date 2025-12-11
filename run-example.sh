#!/usr/bin/env bash

set -ev

cargo build -p clock-example --target wasm32-unknown-unknown --release
cargo run --bin elytra --release -- --device ./target/wasm32-unknown-unknown/release/clock_example.wasm $*