#!/usr/bin/env bash
set -eux

__dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# Install webpack if it's not there
yarn webpack -v || yarn install

yarn build:wasm:dev
yarn build:test-index:federalist
yarn build:js:dev
${__dir}/serve.sh
yarn build:js:dev --watch