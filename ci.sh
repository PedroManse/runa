#! /usr/bin/env bash

set -ex

cargo fmt
cargo check
cargo build
cargo test
cargo clippy --all-targets --all-features
