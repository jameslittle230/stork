export RUST_BACKTRACE := "1"

_default:
    @just --list --unsorted

# Build the indexing binary and JS bundle
build: _build-rust _build-js

# Run Rust and Javascript tests
test: _rust-test _js-test

# Run a Rust benchmark
bench bench_name="":
    cargo criterion --all-features --package stork-lib --plotting-backend=disabled {{bench_name}}

# Remove build artifacts
clean:
    rm -rf dist
    rm -rf pkg
    rm -rf coverage
    rm -rf local-dev/dist
    rm -rf js/dist
    rm -rf target

# Remove build artifacts and downloaded helpers
clean-super: clean
    rm -rf node_modules
    rm -rf local-dev/test-corpora/3b1b/*
    rm -rf local-dev/test-corpora/federalist/*
    rm -rf local-dev/test-indexes/*.st

# Generate Rust package documentation
rustdoc:
    cargo doc --no-deps --open


######################################
## Running the project

alias rbi := run-build-index

run-build-index input="local-dev/test-configs/federalist.toml" output="local-dev/test-indexes/federalist.st":
    cargo run -- build --input {{input}} --output {{output}} --timing

alias rsi := run-search-index

run-search-index query index="local-dev/test-indexes/federalist.st":
    cargo run -- search --index {{index}} --query "{{query}}" --timing

######################################
## Development

dev-init: _yarn
    git submodule init
    git submodule update

dev: dev-init _build-dev _copy-dev-files
    mprocs "just _dev-watch-build" "just _dev-serve" "just _dev-watch-test"

rebuild-dev-indexes:
    rm -rf local-dev/test-indexes/*.st
    just _build-dev-indexes


######################################
## Build command dependencies

_yarn:
    yarn install

_build-rust:
    cargo build --release --all-features

_build-js: _build-wasm
    yarn webpack --config js/webpack/webpack.prod.js

_build-wasm:
    cd stork-wasm; wasm-pack build --target web --out-name stork -- --no-default-features --features="v3"
    wc -c < ./stork-wasm/pkg/stork_bg.wasm

######################################
## Dev command dependencies

_build-dev: _build-rust-dev _build-js-dev

_build-rust-dev:
    cargo build --all-features

_build-js-dev: _build-wasm-dev
    yarn upgrade stork-search
    yarn webpack --config js/webpack/webpack.dev.js

_build-wasm-dev:
    cd stork-wasm; wasm-pack build --target web --out-name stork --dev -- --no-default-features --features="v3"
    wc -c < ./stork-wasm/pkg/stork_bg.wasm

_build-dev-indexes:
    python3 ./scripts/build_dev_indexes.py

_copy-dev-files: _build-dev-indexes
    rm -rf local-dev/dist
    mkdir  local-dev/dist

    cp themes/*.css                 local-dev/dist/
    cp local-dev/site/*             local-dev/dist/
    cp js/dist/*                    local-dev/dist/
    cp local-dev/test-indexes/*.st  local-dev/dist/

_dev-once: dev-init _build-dev _copy-dev-files

_dev-watch-build:
    git ls-files | entr -s "just _build-dev && just _copy-dev-files"

_dev-serve:
    @echo "Open http://127.0.0.1:8025"
    python3 -m http.server --directory ./local-dev/dist 8025

_dev-watch-test:
    git ls-files | entr -s "just test"


######################################
## Test command dependencies

_js-test: _build-wasm _yarn
    yarn jest --coverage

_rust-test:
    cargo test

# format: _yarn
#     cargo fmt
#     yarn prettier --write js/**/*.ts

# lint: _yarn format
#     cargo check
#     cargo clippy --fix --all-targets --all-features -- -D warnings
#     yarn eslint js/**/*.ts


# solo-build-federalist-index:
#     cargo run -q --all-features -- build --input local-dev/test-configs/federalist.toml --output local-dev/test-indexes/federalist.st

# build-federalist-index: build-indexer-dev fetch-test-corpora solo-build-federalist-index

# build-all-indexes: build-federalist-index
#     -cargo run -q --all-features -- build --input local-dev/test-configs/3b1b.toml            --output local-dev/test-indexes/3b1b.st
#     -cargo run -q --all-features -- build --input local-dev/test-configs/beepboop.toml        --output local-dev/test-indexes/beepboop.st
#     -cargo run -q --all-features -- build --input local-dev/test-configs/bowdoin-orient.toml  --output local-dev/test-indexes/bowdoin-orient.st
#     -cargo run -q --all-features -- build --input local-dev/test-configs/federalist-zero.toml --output local-dev/test-indexes/federalist-zero.st





# #############################################
# # Production build settings

# build-indexer:
#     cargo build --release --all-features

# build-wasm:
#     cd stork-wasm && wasm-pack build --target web --out-name stork -- --no-default-features --features="v3"
#     wc -c < ./stork-wasm/pkg/stork_bg.wasm

# build-wasm-all-features:
#     cd stork-wasm && wasm-pack build --target web --out-name stork -- --features="v2, v3"
#     wc -c < ./stork-wasm/pkg/stork_bg.wasm

# solo-build-js:
#     yarn webpack --config js/webpack/webpack.prod.js

# build-js: build-wasm _yarn solo-build-js





# #############################################
# # Development build settings 
# # for the local dev site

# build-indexer-dev:
#     cargo build --all-features

# build-wasm-dev:
#     cd stork-wasm && wasm-pack build --target web --out-name stork --dev -- --no-default-features --features="v3"

# solo-build-js-dev:
#     yarn webpack --config js/webpack/webpack.dev.js

# build-js-dev: build-wasm-dev _yarn solo-build-js-dev

# solo-build-dev-site:
#     rm -rf local-dev/dist
#     mkdir local-dev/dist
#     cp themes/*.css local-dev/dist/
#     cp local-dev/index.html local-dev/dist/
#     cp dist/* local-dev/dist/
#     cp local-dev/test-indexes/*.st local-dev/dist/
#     @echo "You should run \`just serve-dev-site\` in another tab!"

# build-dev-site: build-js-dev build-all-indexes solo-build-dev-site

# build-dev-site-prod: build-js build-all-indexes solo-build-dev-site

# serve-dev-site:
#     @echo "Open http://127.0.0.1:8025"
#     python3 -m http.server --directory ./local-dev/dist 8025





# #############################################
# # CI

# upload ref="":
#     python3 scripts/upload_build_artifacts.py

# bench bench_name="":
#     cargo criterion --all-features --package stork-lib --plotting-backend=disabled --message-format=json {{bench_name}}

# solo-generate-stats:
#     python3 scripts/generate_stats.py

# generate-stats: build-js solo-build-federalist-index solo-generate-stats
    