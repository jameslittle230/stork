set positional-arguments
export RUST_BACKTRACE := "1"

_default:
    @just --list --unsorted







######################################
## Build for release

# Build the project for release
build-release: build-rust-release build-js-release

# Build the Rust components of the project for release
build-rust-release:
    cargo build --release --quiet
    cargo clippy --quiet

# Build and compress the WASM blob
build-wasm-release:
    cd stork-wasm; wasm-pack --quiet build --target web --out-name stork --release
    
    cd stork-wasm/pkg; mv stork_bg.wasm stork_bg_unopt.wasm
    wasm-opt -Os -o stork-wasm/pkg/stork_bg_uncomp.wasm stork-wasm/pkg/stork_bg_unopt.wasm 
    gzip -c stork-wasm/pkg/stork_bg_uncomp.wasm > stork-wasm/pkg/stork_bg.wasm 
    
    -@stat -f 'stork-uncomp.wasm: %z bytes' stork-wasm/pkg/stork_bg_uncomp.wasm
    -@stat -f 'stork.wasm:        %z bytes' stork-wasm/pkg/stork_bg.wasm
    
    mkdir -p js/dist
    cp stork-wasm/pkg/stork_bg_uncomp.wasm js/dist/stork.wasm # TODO: Eventually use the compressed one

# Build the JS components of the project
build-js-release: _yarn
    node build.js







######################################
## Build for development

# Build the project for development
build-dev: _build-rust-dev _build-js-dev

_build-rust-dev:
    cargo build --quiet

# TODO: Document when you might want to do this vs. something else
# _build-wasm-dev:
#     cd stork-wasm; wasm-pack --quiet build --target web --out-name stork --dev
#     cd stork-wasm/pkg; mv stork_bg.wasm stork_bg_uncomp.wasm
#     @echo "Built WASM in dev mode, no sizes reported"

_build-js-dev: _yarn
    node build.js







######################################
## Running the project

alias _rb := run-build

# Build a search index with the default data set
run-build input="dev/configs/federalist.toml" output="dev/indexes/federalist.st":
    cargo run -- build --input {{input}} --output {{output}} --timing

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
    yarn jest --coverage

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
    yarn eslint js/*.ts js/**/*.ts





######################################
## Formatting

fmt-check-all: fmt-check-rust fmt-check-js

fmt-check-rust:
    cargo fmt --check --all

fmt-check-js: _yarn
    yarn prettier js/*.ts js/**/*.ts

fmt-all: fmt-rust fmt-js

fmt-rust:
    cargo fmt --all

fmt-js: _yarn
    yarn prettier --write js/*.ts js/**/*.ts





######################################
## Utilities

_yarn: build-wasm-release
    yarn install --silent
    yarn upgrade stork-search --silent

# Remove build artifacts
clean:
    rm -rf dist
    rm -rf pkg
    rm -rf coverage
    rm -rf dev/dist
    rm -rf js/dist
    rm -rf target

# Remove build artifacts and downloaded helpers
clean-super: clean
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
rebuild-dev-indexes:
    rm -rf dev/indexes/*.st
    just _build-dev-indexes








######################################
## Live Development

# Start a live development session
dev: build-dev _copy-dev-files
    mprocs "just _dev-watch-build" "just _dev-serve" "just _dev-watch-test"

_build-dev-indexes:
    python3 ./scripts/build_dev_indexes.py

_copy-dev-files: _build-dev-indexes
    rm -rf dev/dist
    mkdir  dev/dist

    cp dev/site/*        dev/dist/
    cp js/dist/*         dev/dist/
    cp dev/indexes/*.st  dev/dist/
    cp stork-wasm/pkg/stork_bg_uncomp.wasm  dev/dist/stork.wasm

_dev-watch-build:
    git ls-files | entr -s "just build-dev && just _copy-dev-files"

_dev-serve:
    @echo "Open http://127.0.0.1:8025"
    lsof -t -i tcp:8025 | xargs kill
    python3 -m http.server --directory ./dev/dist 8025

_dev-watch-test:
    git ls-files | entr -s "just test-all"

_dev-watch-build-release:
    git ls-files | entr -s "just build-release"







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
    cd stork-cli;  cargo upgrade -p stork-lib@{{version}}
    cd stork-wasm; cargo upgrade -p stork-lib@{{version}}
    yarn version --new-version {{version}}

tag-version version:
    git tag -a v{{version}} -m "Release version {{version}}"
    git push origin v{{version}}