_default:
  @just --choose

_yarn:
    yarn install






test: _yarn
    cargo test
    yarn jest

format: _yarn
    cargo fmt
    yarn prettier --write js/**/*.ts

lint: _yarn format
    cargo check
    ./scripts/clippy.sh
    yarn eslint js/**/*.ts

clean:
    rm -rf dist
    rm -rf pkg
    rm -rf coverage
    rm -rf target

super-clean: clean
    rm -rf node_modules
    rm -rf local-dev/test-corpora/3b1b/*
    rm -rf local-dev/test-corpora/federalist/*






fetch-federalist-corpus:
    rm -rf local-dev/test-corpora/federalist
    git clone git@github.com:jameslittle230/federalist.git local-dev/test-corpora/federalist

fetch-3b1b-corpus:
    @echo "fetch-3b1b-corpus WIP"

solo-build-federalist-index:
    cargo run --all-features -- build --input local-dev/test-configs/federalist.toml --output local-dev/test-indexes/federalist.st

build-federalist-index: build-indexer-dev fetch-federalist-corpus solo-build-federalist-index

build-all-indexes: build-federalist-index fetch-3b1b-corpus
    -cargo run -q --all-features -- build --input local-dev/test-configs/3b1b.toml           --output local-dev/test-indexes/3b1b.st
    -cargo run -q --all-features -- build --input local-dev/test-configs/beepboop.toml       --output local-dev/test-indexes/beepboop.st
    -cargo run -q --all-features -- build --input local-dev/test-configs/bowdoin-orient.toml --output local-dev/test-indexes/bowdoin-orient.st
    -cargo run -q --all-features -- build --input local-dev/test-configs/web-pages.toml      --output local-dev/test-indexes/web-pages.st





#############################################
# Production build settings

build-indexer:
    cargo build --release --all-features

build-wasm:
    wasm-pack build --target web --out-name stork -- --no-default-features

solo-build-js:
    yarn webpack --config webpack.prod.js

build-js: build-wasm _yarn solo-build-js





#############################################
# Development build settings 
# for the local dev site

build-indexer-dev:
    cargo build --all-features

build-wasm-dev:
    wasm-pack build --target web --out-name stork --dev -- --no-default-features

solo-build-js-dev:
    yarn webpack --config webpack.dev.js

build-js-dev: build-wasm-dev _yarn solo-build-js-dev

solo-build-dev-site:
    rm -rf local-dev-dist
    mkdir local-dev-dist
    cp themes/*.css local-dev-dist/
    cp local-dev/index.html local-dev-dist/
    cp dist/* local-dev-dist/
    cp local-dev/test-indexes/*.st local-dev-dist/
    @echo "You should run ./scripts/serve.sh in another tab!"

build-dev-site: build-js-dev build-all-indexes solo-build-dev-site

build-dev-site-prod: build-js build-all-indexes solo-build-dev-site
    