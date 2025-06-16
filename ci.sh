#! /usr/bin/env bash
set -ex
cargo build
cargo fmt
cargo clippy --all-targets --all-features -- \
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
