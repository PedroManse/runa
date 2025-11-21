#! /usr/bin/env bash
set -e

if [ -n "$FIX" ] && [ "$FIX" != "0" ] ; then
	fix="--fix"
fi

if [ -n "$DIRTY" ] && [ "$DIRTY" != "0" ] ; then
	allow_dirty="--allow-dirty"
fi

set -x

cargo clippy $fix $allow_dirty --all-targets --all-features -- \
	-Dclippy::perf \
	-Dclippy::style \
	-Dclippy::pedantic \
	-Wclippy::unwrap_used \
	-Wclippy::missing_errors_doc
