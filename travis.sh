#!/usr/bin/env bash

set -o errexit

travis-cargo build -- --features i3-next
travis-cargo --only nightly doc -- --features dox
rustup component add rustfmt
rustfmt --version
cargo fmt -- --check
