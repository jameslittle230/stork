#!/usr/bin/env bash
# This script builds the entire project, creating build artifacts for both
# the stork binary and the Javascript library. It'll also build the Federalist
# Papers index, just for kicks.
set -eux

# First, build the Rust components of the project...
cargo build --release
yarn build:wasm:prod # ...including the WASM library.

# Install JS dependencies, now that the WASM library has been built
yarn install

## Build the text index and the JS library
yarn build:js:prod

# Build the Federalist Papers index, downloading the test corpus if needed
if [ ! -f "./test/federalist/federalist-1.txt" ]; then
    ./scripts/download_test_corpora.sh
fi

yarn build:test-index:federalist