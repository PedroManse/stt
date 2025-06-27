#! /usr/bin/env bash

if [ "$1" = "--allow-dirty" ] || [ "$2" = "--allow-dirty" ] ; then allow_dirty="--allow-dirty" ; fi
if [ "$1" = "--fix" ] || [ "$2" = "--fix" ] ; then fix="--fix" ; fi
set -ex

cd lang

cargo build
cargo fmt
cargo clippy $fix $allow_dirty --all-targets --all-features -- \
	-Dclippy::perf \
	-Dclippy::style \
	-Wclippy::pedantic \
	-Aclippy::unnested_or_patterns \
	-Aclippy::wildcard_imports \
	-Aclippy::enum_glob_use \
	-Aclippy::too_many_lines \
	-Aclippy::match_same_arms \
	-Aclippy::unnecessary_wraps \
	-Aclippy::missing_errors_doc \
	-Aclippy::cast_sign_loss \
	-Aclippy::cast_possible_wrap \
	-Aclippy::cast_possible_truncation
# should remove the last 4
cargo test

cd ../interpreter

cargo build
cargo fmt
cargo clippy $fix $allow_dirty --all-targets --all-features -- \
	-Dclippy::perf \
	-Dclippy::style \
	-Wclippy::pedantic \
	-Aclippy::unnested_or_patterns \
	-Aclippy::wildcard_imports \
	-Aclippy::enum_glob_use \
	-Aclippy::too_many_lines \
	-Aclippy::match_same_arms \
	-Aclippy::unnecessary_wraps \
	-Aclippy::missing_errors_doc \
	-Aclippy::cast_sign_loss \
	-Aclippy::cast_possible_wrap \
	-Aclippy::cast_possible_truncation
# should remove the last 4
cargo test
