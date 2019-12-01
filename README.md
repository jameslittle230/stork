# Stork

An impossibly fast web search solution.

Stork builds a search index of your site's content during the build step of site generation. Then it uses a web-assembly frontend module to download and quickly search for results on your site. No computation takes place during a search operation.

Currently in development by [James Little](https://jameslittle.me)

# Developing

Building the WebAssembly project:

```
$ wasm-pack build --target no-modules
```

Building an index from `federalist.toml`

```
$ cargo run -- --build test/federalist.toml
```

Searching from the command line:

```
$ cargo run -- --search out.stork liberty
```
