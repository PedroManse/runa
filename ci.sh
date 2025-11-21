#! /usr/bin/env bash

set -ex
cargo fmt
cargo check
cargo build
cargo test

set +x
bash lint.sh $@
set +e
