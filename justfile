set positional-arguments
export RUST_BACKTRACE := "1"

_default:
    @just --list --unsorted







######################################
## Build for release

# Build the project for release
build-release: build-rust-release build-js-release

build-ts-bindings:
    cargo test --package stork-lib --quiet --all-features export_bindings_ 2> /dev/null # Build TS bindings

# Build the Rust components of the project for release
build-rust-release: build-ts-bindings
    cargo build --release --quiet
    cargo clippy --quiet

# Build and compress the WASM blob
build-wasm-release: build-ts-bindings
    cd stork-wasm; wasm-pack --quiet build --target web --out-name stork --release
    mv stork-wasm/pkg/stork_bg.wasm stork-wasm/pkg/stork_bg_unopt.wasm
    wasm-opt -Os -o stork-wasm/pkg/stork.wasm stork-wasm/pkg/stork_bg_unopt.wasm

# Build the JS components of the project
build-js-release: _yarn
    cd js; yarn --silent run tsup --config build.js --silent
    cp stork-wasm/pkg/stork.wasm js/dist/stork.wasm
    @ruby -e "puts \"stork.wasm: #{File.size(\"js/dist/stork.wasm\") / 1000} KB\""
    @ruby -e "puts \"stork.js:   #{File.size(\"js/dist/stork.js\") / 1000} KB\""







######################################
## Build for development

# Build the project for development
build-dev: build-rust-dev build-js-release

build-rust-dev:
    cargo build --quiet







######################################
## Running the project

alias _rb := run-build

# Build a search index with the default data set
run-build input="dev/configs/federalist.toml" output="dev/indexes/federalist.st":
    cargo run --all-features -- build --input {{input}} --output {{output}} --timing

alias _rs := run-search

# Run a search on the default development index
run-search query index="dev/indexes/federalist.st":
    cargo run -- search --index {{index}} --query "{{query}}" --timing







######################################
## Testing

# Run all project tests
test-all: test-rust test-js

# Run Rust benchmarks
bench bench_name="":
    cargo criterion --all-features --package stork-lib {{bench_name}}

# Run JS tests
test-js: _yarn
    cd js; yarn jest --coverage

# Run Rust tests
test-rust:
    cargo test







######################################
## Linting

lint-all: lint-rust lint-js

lint-rust:
    cargo check
    cargo clippy --all-targets --all-features -- -D warnings

lint-js: _yarn
    cd js; yarn eslint js/*.ts js/**/*.ts





######################################
## Formatting

fmt-check-all: fmt-check-rust fmt-check-js

fmt-check-rust:
    cargo fmt --check --all

fmt-check-js: _yarn
    cd js; yarn prettier js/*.ts js/**/*.ts

fmt-all: fmt-rust fmt-js

fmt-rust:
    cargo fmt --all

fmt-js: _yarn
    cd js; yarn prettier --write js/*.ts js/**/*.ts





######################################
## Utilities

_yarn: build-wasm-release
    cd js; yarn install --silent
    cd js; yarn upgrade stork-search --silent

# Remove build artifacts
clean:
    rm -rf dist
    rm -rf pkg
    rm -rf coverage
    rm -rf dev/dist
    rm -rf js/dist
    rm -rf target

# Remove build artifacts and downloaded helpers
deep-clean: clean
    rm -rf node_modules
    rm -rf dev/documents/3b1b/*
    rm -rf dev/documents/federalist/*
    rm -rf dev/indexes/*.st

# Generate Rust package documentation
rustdoc:
    cargo doc --no-deps --open

# Update Git submodules
submodules:
    git submodule init
    git submodule update

# Build the development indexes
rebuild-dev-indexes: && _build-dev-indexes
    mkdir -p dev/indexes
    rm -rf dev/indexes/*.st








######################################
## Live Development

# Start a live development session
dev: build-dev _copy-dev-files
    open http://127.0.0.1:8025
    mprocs --config dev/mprocs.yml

# Command to list all files that should be watched for changes
_list_watched_files:
    git ls-files js/src stork-*/src dev/site

_build-dev-indexes:
    python3 scripts/build_dev_indexes.py

_copy-dev-files: _build-dev-indexes
    rm -rf dev/dist
    mkdir  dev/dist

    cp dev/site/*        dev/dist/
    cp js/dist/*         dev/dist/
    cp dev/indexes/*.st  dev/dist/
    cp stork-wasm/pkg/stork_bg_uncomp.wasm  dev/dist/stork.wasm

_dev-watch-build:
    just _list_watched_files | entr -s "just build-dev && just _copy-dev-files"

_dev-serve:
    @echo "Open http://127.0.0.1:8025"
    lsof -t -i tcp:8025 | xargs kill
    python3 -m http.server --directory ./dev/dist 8025

_dev-watch-test:
    just _list_watched_files | entr -s "just test-all"

_dev-watch-build-release:
    just _list_watched_files | entr -s "just build-release"







######################################
# Releasing

generate-stats: build-js-release rebuild-dev-indexes
    python3 scripts/generate_stats.py

print-deploy:
    pandoc -o ~/Desktop/deploy.pdf docs/deploy.md --pdf-engine=/Library/TeX/texbin/pdflatex -V geometry:margin=0.75in
    open ~/Desktop/deploy.pdf

# Set the versions of the crates and the JS project
set-versions version:
    cargo set-version --workspace {{version}}
    cd stork-cli;  cargo upgrade --offline -p stork-lib@{{version}}
    cd stork-wasm; cargo upgrade --offline -p stork-lib@{{version}}
    cd js; yarn version --new-version {{version}}

tag-version version:
    git tag -a v{{version}} -m "Release version {{version}}"
    git push origin v{{version}}

publish:
    cd stork-lib; cargo publish
    cd stork-cli; cargo publish --all-features
    cd js; yarn publish