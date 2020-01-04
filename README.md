# Stork

An impossibly fast web search solution.

Stork is two things. First, it's an indexer: it turns your loosely-structured content and builds a search index from that content. Second, it's a Javascript + WebAssembly frontend for that index; Stork will download the index, search through it, and display the best results immediately to your user, as they type. The precomputed index and WebAssembly frontend module make the entire Stork engine very good, and _very_ fast.

Currently in development by [James Little](https://jameslittle.me)

# Getting Started

Let's put a search box online that searches within the text of the [Federalist Papers](https://www.youtube.com/watch?v=DPgE7PNzXag).

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

**Step 1: Include the HTML**

Stork hooks into existing HTML that you include on your page. Each Stork instance has to have an input hook and a results list; those two elements should be placed in a wrapper, though the wrapper is optional.

The input hook should have the `data-stork="federalist"` attribute, where `federalist` is the name with which you register that search instance. (This way, you can have multiple, independent search boxes on a page, all pointing to different instances.) It doesn't have to be `federalist` -- you can change it to whatever you want.

The results list should be an empty `<div>` tag with the attribute `data-stork="federalist-results"`. Again, here, you can change `federalist` to whatever you want.

The classes in the example above (`stork-input`, `stork-output`) are for the theme. Most Stork themes assume the format above; the theme documentation will tell you if it requires something different. You can also design your own theme, at which point the styling and class names are up to you.

**Step 2: Include the Javascript**

You need to include `stork.js`, which you can either load from the Stork CDN (optimal) or host yourself (not optimal). This will load the Stork WebAssembly blob and create the Stork object, which will allow for registering and configuring indices.

Then, you should register at least one index:

```javascript
stork.register("federalist", "http://files.stork-search.net/federalist.st");
```

The search index you build needs to be stored somewhere with a public URL. To register

This registers the index stored at `http://files.stork-search.net/federalist.st` under the name `federalist`; the `data-stork` attributes in the HTML will hook into this name.

Finally, you can set some configuration options for how your search bar will interact with the index and with the page.

# Building your own index

You probably don't want to add an interface to your own website that lets you search through the Federalist papers. Here's how to make your search bar yours.

The search index is based on a document structure: you give Stork a list of documents on disk and include some metadata about those documents, and Stork will build its search index based on the contents of those documents.

To begin, you need a configuration file that describes, among other things, that list of files:

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

[output]
filename = "federalist.st"
```

This TOML file describes the base directory of all your documents, then lists out each document along with the web URL at which that document will be found, along with that document's title.

# Going Further

You can read more documentation and learn more about customization at the project's website: <https://stork-search.net>.

# Developing

Dependencies include:

- Rust, installed from rustup
- wasm-pack
- npm

**Build the project:**

```
$ cargo make build
```

This runs some sub-build steps:

- format
- build-wasm (which compiles rust into webassembly)
- build-js (which runs webpack on the various JS/WASM files created by the prior step)
- build-indexer (which builds the command line indexing interface)

Once you've built the project, you can **serve the web parts from the `dist/` directory**:

```
$ python3 -m http.server --directory dist/
```

Or you can use the HTTP server of your choice.

**Building an index from `federalist.toml`**

```
$ cargo run -- --build test/federalist.toml
```

**Searching from the command line**:

```
$ cargo run -- --search federalist.st liberty
```
