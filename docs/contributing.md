# Contributing

Hi! Thank you for your interest in contributing to Stork!

## Development

To build Stork, you'll need:

- The stable toolchain of [Rust](https://rust-lang.org)
- [wasm-pack](https://github.com/rustwasm/wasm-pack)
- [yarn](https://yarnpkg.com)

Optionally:

- [just](https://github.com/casey/just)
- [mprocs](https://github.com/pvolok/mprocs)

## Project Structure

- `js` holds the Javascript source. Run `build.js` (in the root) with node to build the Javascript part of the project.
- `stork-cli` and `stork-wasm` are crates that depend on `stork-lib`.
- `dev` holds assets needed to build some test indexes, as well as a test website.

## Building the Project

`just` is a task runner, similar to `make`. Running `just` alone will output all the possible scripts as well as a short description of what they do. Here are the hits:

- `just build-release` will build the project for release.
- `just dev` will set up the development server with live-reload capabilities.
- `just submodules` will initialize and fetch the contents of Git Submodules. Stork uses Git Submodules to fill the contents of /dev/documents with test documents to be indexed.
- `just _rb` will build a search index for the federalist papers.
- `just _rs <query>` will search that index.
- `just test-all` runs all the tests.

## Finding build artifacts

After you run `just build-release`, there will be three artifacts that you might be interested in.

- `/target/release/stork` is the compiled indexer binary
- `/js/dist/stork_bg.wasm` is the WASM blob.
- `/js/dist/stork.js` is the minified JS bundle.
- `/js/dist/stork.css` is the CSS that makes your input tag look pretty.

## Glossary

```text
                                  ┌───────────────────── ARENA Y ──┐   ┌──────── ARENA X ─┐
                  ┌────────┐      │                  ┌─────────┐   │   │ ┌─────────┐      │
              ┌──▶│  5, 6  │      │              ┌──▶│  9, 10  │───┼a──┼▶│   ???   │      │
 ┌────────┐  b│   └────────┘      │ ┌────────┐  a│   └─────────┘   │   │ └─────────┘      │
 │  1, 2  │───┤                ┌──┼▶│  7, 8  │───┤                 │   │                  │
 └────────┘  a│   ┌────────┐  a│  │ └────────┘  b│   ┌─────────┐   │   └──────────────────┘
              └──▶│  3, 4  │───┤  │              └──▶│ 13, 14  │   │
                  └────────┘   │  │                  └─────────┘   │
                              b│  └────────────────────────────────┘
                               │
                               │    ┌────────┐
                               └───▶│ 11, 12 │
                                    └────────┘
```

- **Index**
  - A data structure that contains all the information needed to enable searching through content. Indexes can be stored and used in memory, and they can be written to disk as a series of files.
- **Tree**
  - The primary data structure for the V4 search index. Made up of a hashmap of **arenas**.
- **Arena**
  - A subtree within a V4 search index. Contains a set of **nodes**. An arena can be **loaded** or **unloaded** in memory. Identified by an **ArenaId**.
- **Content Slice**
  - A contiguous portion of text content that is searchable and belongs to an associated document.
- **Chunk**
  - A set of data that is saved to a file. Chunks are identified by a **ChunkId**, and contain zero or more **arenas** and zero or more **Content slices**. When a chunk is downloaded, all the arenas and content slices are loaded into the in-memory index.
  - There are two types of chunks - a **Root Chunk** and a **Branch Chunk**. The root chunk contains additional metadata.
