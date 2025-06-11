#! /usr/bin/env bash
cargo build
cargo fmt
cargo clippy --all-targets --all-features
cargo test
