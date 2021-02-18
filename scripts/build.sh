#!/usr/bin/env bash
set -eux

cargo build --release
yarn webpack -v || yarn install
yarn build:wasm:prod
yarn build:test-index:federalist
yarn build:js:prod