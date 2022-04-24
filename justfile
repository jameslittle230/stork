_default:
  @just --choose

_yarn:
    yarn install




_js-test: build-wasm _yarn
    yarn jest --coverage

_rust-test:
    cargo test

test: _js-test _rust-test

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




fetch-test-corpora:
    git submodule init
    git submodule update

solo-build-federalist-index:
    cargo run -q --all-features -- build --input local-dev/test-configs/federalist.toml --output local-dev/test-indexes/federalist.st

build-federalist-index: build-indexer-dev fetch-test-corpora solo-build-federalist-index

build-all-indexes: build-federalist-index
    -cargo run -q --all-features -- build --input local-dev/test-configs/3b1b.toml            --output local-dev/test-indexes/3b1b.st
    -cargo run -q --all-features -- build --input local-dev/test-configs/beepboop.toml        --output local-dev/test-indexes/beepboop.st
    -cargo run -q --all-features -- build --input local-dev/test-configs/bowdoin-orient.toml  --output local-dev/test-indexes/bowdoin-orient.st
    -cargo run -q --all-features -- build --input local-dev/test-configs/federalist-zero.toml --output local-dev/test-indexes/federalist-zero.st





#############################################
# Production build settings

build-indexer:
    cargo build --release --all-features

build-wasm:
    cd stork-wasm && wasm-pack build --target web --out-name stork -- --no-default-features --features="v3"
    wc -c < ./stork-wasm/pkg/stork_bg.wasm

build-wasm-all-features:
    cd stork-wasm && wasm-pack build --target web --out-name stork -- --features="v2, v3"
    wc -c < ./stork-wasm/pkg/stork_bg.wasm

solo-build-js:
    yarn webpack --config webpack.prod.js

build-js: build-wasm _yarn solo-build-js





#############################################
# Development build settings 
# for the local dev site

build-indexer-dev:
    cargo build --all-features

build-wasm-dev:
    cd stork-wasm && wasm-pack build --target web --out-name stork --dev -- --no-default-features --features="v3"

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
    @echo "You should run \`just serve-dev-site\` in another tab!"

build-dev-site: build-js-dev build-all-indexes solo-build-dev-site

build-dev-site-prod: build-js build-all-indexes solo-build-dev-site

serve-dev-site:
    @echo "Open http://127.0.0.1:8025"
    python3 -m http.server --directory ./local-dev-dist 8025





#############################################
# CI

upload ref="":
    python3 scripts/upload_build_artifacts.py

bench bench_name="":
    cargo criterion --package stork-lib --plotting-backend=disabled --message-format=json {{bench_name}}

solo-generate-stats:
    python3 scripts/generate_stats.py

generate-stats: build-js solo-build-federalist-index solo-generate-stats
    