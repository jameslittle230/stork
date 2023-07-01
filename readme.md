# Stork

<table><tr><td>

**Project update**: [I'm winding down my work with Stork.](https://github.com/jameslittle230/stork/discussions/360)

Thanks to everyone who enjoyed using Stork over the past few years!  
-James
  
</td></tr></table>

---

Impossibly fast web search, made for static sites.

[![Crates.io](https://img.shields.io/crates/v/stork-search)](https://crates.io/crates/stork-search)
[![Codecov](https://img.shields.io/codecov/c/gh/jameslittle230/stork)](https://codecov.io/gh/jameslittle230/stork)
![GitHub branch checks state](https://img.shields.io/github/checks-status/jameslittle230/stork/master)

Stork is a library for creating beautiful, fast, and accurate full-text search interfaces on the web.

It comes in two parts. First, it's a **command-line tool** that indexes content and creates a search index file that you can upload to a web server. Second, it's a **Javascript library** that uses that index file to build an interactive search interface that displays optimal search results immediately to your user, as they type.

Stork is built with Rust, and the Javascript library uses WebAssembly behind the scenes. It's easy to get started and is even easier to customize so it fits your needs. It's perfect for Jamstack sites and personal blogs, but can be used wherever you need an interactive search bar.

Currently in development by [James Little](https://jameslittle.me)

![Gif of Stork in Action](https://files.stork-search.net/marketing/1.0.0-video.gif)

## Getting Started

Let's put a search box online that searches within the text of the [Federalist Papers](https://www.youtube.com/watch?v=DPgE7PNzXag).

> **See this demo live at <https://stork-search.net>.**

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>Federalist Search</title>
  </head>
  <body>
    <div class="stork-wrapper">
      <input data-stork="federalist" class="stork-input" />
      <div data-stork="federalist-output" class="stork-output"></div>
    </div>
    <script src="https://files.stork-search.net/stork.js"></script>
    <script>
      stork.register(
        "federalist",
        "http://files.stork-search.net/federalist.st"
      );
    </script>
  </body>
</html>
```

### Step 1: Include the HTML

Stork hooks into existing HTML that you include on your page. Each Stork instance has to have an input hook and a results list; those two elements should be placed in a wrapper, though the wrapper is optional.

The input hook should have the `data-stork="federalist"` attribute, where `federalist` is the name with which you register that search instance. (This way, you can have multiple, independent search boxes on a page, all pointing to different instances.) It doesn't have to be `federalist` -- you can change it to whatever you want.

The results list should be an empty `<div>` tag with the attribute `data-stork="federalist-results"`. Again, here, you can change `federalist` to whatever you want.

The classes in the example above (`stork-input`, `stork-output`) are for the theme. Most Stork themes assume the format above; the theme documentation will tell you if it requires something different. You can also design your own theme, at which point the styling and class names are up to you.

### Step 2: Include the Javascript

You need to include `stork.js`, which you can either load from the Stork CDN or host yourself. This will load the Stork WebAssembly blob and create the Stork object, which will allow for registering and configuring indices.

Then, you should register at least one index:

```javascript
stork.register("federalist", "http://files.stork-search.net/federalist.st");
```

The search index you build needs to be stored somewhere with a public URL. To register

This registers the index stored at `http://files.stork-search.net/federalist.st` under the name `federalist`; the `data-stork` attributes in the HTML will hook into this name.

Finally, you can set some configuration options for how your search bar will interact with the index and with the page.

## Building your own index

You probably don't want to add an interface to your own website that lets you search through the Federalist papers. Here's how to make your search bar yours.

To build an index, you need the Stork executable on your computer, which you can install at the [latest Github release](https://github.com/jameslittle230/stork/releases) or by running `cargo install stork-search --locked` if you have a Rust toolchain installed.

The search index is based on a document structure: you give Stork a list of documents on disk and include some metadata about those documents, and Stork will build its search index based on the contents of those documents.

First, you need a configuration file that describes, among other things, that list of files:

```toml
[input]
base_directory = "test/federalist"
files = [
    {path = "federalist-1.txt", url = "/federalist-1/", title = "Introduction"},
    {path = "federalist-2.txt", url = "/federalist-2/", title = "Concerning Dangers from Foreign Force and Influence"},
    {path = "federalist-3.txt", url = "/federalist-3/", title = "Concerning Dangers from Foreign Force and Influence 2"},
    {path = "federalist-4.txt", url = "/federalist-4/", title = "Concerning Dangers from Foreign Force and Influence 3"},
    {path = "federalist-5.txt", url = "/federalist-5/", title = "Concerning Dangers from Foreign Force and Influence 4"},
    {path = "federalist-6.txt", url = "/federalist-6/", title = "Concerning Dangers from Dissensions Between the States"},
    {path = "federalist-7.txt", url = "/federalist-7/", title = "Concerning Dangers from Dissensions Between the States 2"},
    {path = "federalist-8.txt", url = "/federalist-8/", title = "The Consequences of Hostilities Between the States"},
    {path = "federalist-9.txt", url = "/federalist-9/", title = "The Union as a Safeguard Against Domestic Faction and Insurrection"},
    {path = "federalist-10.txt", url = "/federalist-10/", title = "The Union as a Safeguard Against Domestic Faction and Insurrection 2"}
]
```

This TOML file describes the base directory of all your documents, then lists out each document along with the web URL at which that document will be found, along with that document's title.

From there, you can build your search index by running:

```bash
$ stork build --input federalist.toml --output federalist.st
```

This will create a new file at `federalist.st`. You can search through it with the same command line tool:

```bash
$ stork search --index federalist.st --query "liberty"
```

To embed a Stork search interface on your website, first upload the index file to your web server, then pass its URL to the `stork.register()` function in your web page's Javascript.

## Going further

You can read more documentation and learn more about customization at the project's website: <https://stork-search.net>.

# Development

To build Stork, you'll need:

- [Rust](https://www.rust-lang.org), using the stable toolchain
- [wasm-pack](https://github.com/rustwasm/wasm-pack)
- [yarn](https://yarnpkg.com)
- [Just](https://github.com/casey/just) if you want to use the same build scripts I do (otherwise you can read the Justfile and run the scripts manually)

The repository is structured like a [typical Cargo workspace](https://doc.rust-lang.org/cargo/reference/workspaces.html), with some modifications.

- The `stork-*` directories hold Rust packages. `stork-cli` and `stork-wasm` are the top-level packages; everything else is a dependency.
- `js` holds the Javascript source code.
- `test-assets` holds binary assets required by Stork's functional tests.
- `local-dev` holds configuration files, corpora, and index files required to build and run the test webpage used for local development.

You can build the project using either the Rust entrypoint or the Javascript entrypoint (build instructions are listed below). After you've built the project, you'll see three more directories:

- `target` holds the output binary build artifacts
- `pkg` holds intermediate WASM build artifacts
- `dist` holds the final build artifacts for the web.

If you're interested in extracting the final Stork build artifacts, you can extract the following files after building the project with `yarn build`:

- `/target/release/stork`
- `/dist/stork.js`
- `/dist/stork.wasm`

## Building the project for production

- `just build-indexer` will build the indexer binary to `target/release/stork`
- `just build-js` will build the WASM binary and the Javascript bridging code to the `dist` directory
- `just build-federalist-index` will build the federalist.st index file that's referenced throughout the project. It will output to `local-dev/test-indexes/federalist.st`.

### Building the project for development

- `just build-indexer-dev` will build the indexer binary
- `cargo run -- <CLI OPTIONS>` will run the indexer binary
- `just build-dev-site` will build the WASM and Javascript bridge code, build the federalist.st index, and package the development site
- `./scripts/serve.sh` will serve the development site

Take a look at the project's Justfile for more available scripts.
