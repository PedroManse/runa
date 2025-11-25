#! /usr/bin/env bash

set -ex
if [ -n "$FIX" ] && [ "$FIX" != "0" ] ; then
	fix="--fix"
fi

if [ -n "$DIRTY" ] && [ "$DIRTY" != "0" ] ; then
	allow_dirty="--allow-dirty"
fi

cargo fmt
cargo check
cargo build
cargo test
cargo clippy $fix $allow_dirty --all-targets --all-features
