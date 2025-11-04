#!/bin/bash
set -e

cargo check \
	&& cargo fmt --all -- --check \
	&& cargo clippy -- -D warnings \
	&& cargo audit
