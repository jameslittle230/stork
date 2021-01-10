#!/usr/bin/env bash
set -eux

yarn webpack -v || yarn install
cargo build --release
yarn build:wasm:prod
yarn build:test-index:federalist
yarn build:js:prod