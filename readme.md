<div align="center">

# 🔎 Stork Search

**A library for creating beautiful, fast, and accurate full-text search interfaces on the web.**

[![Crates.io](https://img.shields.io/crates/v/stork-search)](https://crates.io/crates/stork-search?style=plastic)
![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/jameslittle230/stork/ci-on-push.yml?branch=master&style=plastic)

### [Website](https://stork-search.net/docs/install) • [Documentation](https://stork-search.net/docs/install) • [Demo](https://codepen.io/littleguy230/pen/oNBJBmK) • [Donate](https://ko-fi.com/jameslittle230) • [Chat](https://stork-search.net/chat) • [Stickers](https://stork-search.net/sticker)

![Gif of Stork in Action](https://files.stork-search.net/marketing/1.0.0-video.gif)

</div>

Stork Search is two pieces of software. First, it's a **command-line tool** that indexes content and creates a search index. Second, it's a **Javascript library** that uses that index file to build an interactive search interface that displays optimal search results immediately to your user as they type.

Stork is built with Rust, and the Javascript library uses WebAssembly behind the scenes. It's easy to get started and is even easier to customize so it fits your needs. It's perfect for Jamstack sites and personal blogs, but can be used wherever you need an interactive search bar.

## Quickstart

Let's put a search box online that searches within the text of the [Federalist Papers](https://www.youtube.com/watch?v=DPgE7PNzXag).

> **→ [See the final result live on CodePen]().**

### Step 1: Create a search index

A Stork search index begins with a configuration file defining where to find the content that can be indexed. Stork can index content from the filesystem, from the internet, or embedded directly in the configuration file.

> **→ [Read the documentation about the Stork configuration file]()**

To index the Federalist Papers, we can set up a TOML configuration file that retrieves them from the web:

```toml
[[input.files]]
title = "1: General Introduction"
url = "https://federalist.stork-search.net/1.html"

[[input.files]]
title = "2-5: Concerning Dangers from Foreign Force and Influence"
url = "https://federalist.stork-search.net/2-5.html"

[[input.files]]
title = "6-7: Concerning Dangers from Dissentions Between the States"
url = "https://federalist.stork-search.net/6-7.html"

[[input.files]]
title = "8: The Consequences of Hostilities Between the States"
url = "https://federalist.stork-search.net/8.html"

[[input.files]]
title = "9-10: The Union as a Safeguard Against Domestic Faction and Insurrection"
url = "https://federalist.stork-search.net/9-10.html"

[[input.files]]
title = "11: The Utility of the Union in Respect to Commercial Relations and a Navy"
url = "https://federalist.stork-search.net/11.html"

[[input.files]]
title = "12: The Utility of the Union in Respect to Revenue"
url = "https://federalist.stork-search.net/12.html"
```

Stork configuration files can be written in TOML or JSON. There are a lot more settings than described here, but this configuration file relies on many implicit defaults.

<details>
<summary><b>View this configuration file in JSON</b></summary>

Stork will automatically detect whether a given index is TOML or JSON and parse it appropriately.

```json
{
  "input": {
    "files": [
      {
        "title": "1: General Introduction",
        "url": "https://federalist.stork-search.net/1.html"
      },
      {
        "title": "2-5: Concerning Dangers from Foreign Force and Influence",
        "url": "https://federalist.stork-search.net/2-5.html"
      },
      {
        "title": "6-7: Concerning Dangers from Dissentions Between the States",
        "url": "https://federalist.stork-search.net/6-7.html"
      },
      {
        "title": "8: The Consequences of Hostilities Between the States",
        "url": "https://federalist.stork-search.net/8.html"
      },
      {
        "title": "9-10: The Union as a Safeguard Against Domestic Faction and Insurrection",
        "url": "https://federalist.stork-search.net/9-10.html"
      },
      {
        "title": "11: The Utility of the Union in Respect to Commercial Relations and a Navy",
        "url": "https://federalist.stork-search.net/11.html"
      },
      {
        "title": "12: The Utility of the Union in Respect to Revenue",
        "url": "https://federalist.stork-search.net/12.html"
      }
    ]
  }
}
```

</details>

<details>
<summary><b>View a configuration file that indexes content from the filesystem</b></summary>

In the previous configuration file, the `url` key is doing double-duty: it's telling Stork where to find the source HTML file to be indexed, _and_ it's saying where the user should navigate to when they click on that search result. (If you want the source and destination URLs to be different, you can use the [src_url]() key to enable that.)

Here, the `url` key only describes the destination, and the `path` key indicates that the source HTML file can be found on the filesystem.

The paths described this configuration file are relative to the location where you run the `stork` binary.

```toml
[[input.files]]
title = "1: General Introduction"
path = "1.html"
url = "https://federalist.stork-search.net/1.html"

[[input.files]]
title = "2-5: Concerning Dangers from Foreign Force and Influence"
path = "2-5.html"
url = "https://federalist.stork-search.net/2-5.html"

[[input.files]]
title = "6-7: Concerning Dangers from Dissentions Between the States"
path = "6-7.html"
url = "https://federalist.stork-search.net/6-7.html"

[[input.files]]
title = "8: The Consequences of Hostilities Between the States"
path = "8.html"
url = "https://federalist.stork-search.net/8.html"

[[input.files]]
title = "9-10: The Union as a Safeguard Against Domestic Faction and Insurrection"
path = "9-10.html"
url = "https://federalist.stork-search.net/9-10.html"

[[input.files]]
title = "11: The Utility of the Union in Respect to Commercial Relations and a Navy"
path = "11.html"
url = "https://federalist.stork-search.net/11.html"

[[input.files]]
title = "12: The Utility of the Union in Respect to Revenue"
path = "12.html"
url = "https://federalist.stork-search.net/12.html"
```

</details>

After [installing the Stork command-line tool]() and saving the file, run the following command to build a search index:

```sh
$ stork build --input federalist.toml --output federalist.st
```

Stork wrote a search index file named `federalist.st` to your filesystem. You can test it out with the same command-line tool by performing a search:

```sh
$ stork search --index federalist.st --query "liberty"
```

### Step 2: Embed it on a webpage

To build an interactive, online search interface, you can use the Stork Javascript library to load your search index and attach it to HTML on your webpage.

You'll need to take the output file generated in the previous step and make it accessible on a web server. I've already done that and uploaded the search index to `https://files.stork-search.net/federalist.st`.

Stork looks for the `data-stork` attributes on two tags: an `<input>` tag where your users will type their search query, and a `<div>` tag where Stork will render the search results. Here, we're setting up our input and output elements with the name "federalist"—we'll use that name later to point the Javascript library at the correct HTML tags.

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>Search</title>
  </head>
  <body>
    <input data-stork="federalist" />
    <div data-stork="federalist-output"></div>
  </body>
</html>
```

By default, Stork's output is completely unstyled, letting you customize the output however you want. However, Stork also provides a "base" theme that can be customized using CSS variables. Before we continue, we'll add some classes, structure, and a `<link>` tag to load up Stork's "base" theme.

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>Search</title>
    <link rel="stylesheet" href="https://files.stork-search.net/basic.css" />
  </head>
  <body>
    <div class="stork-wrapper">
      <input data-stork="federalist" class="stork-input" />
      <div data-stork="federalist-output" class="stork-output"></div>
    </div>
  </body>
</html>
```

> **→ [Learn more about theming Stork's UI]()**

Finally, we'll load the Stork Javascript library and register our search index:

```html
<!DOCTYPE html>
<html lang="en">
  <head>
    <title>Search</title>
    <link rel="stylesheet" href="https://files.stork-search.net/basic.css" />
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
        "https://files.stork-search.net/federalist.st",
        {}
      );
    </script>
  </body>
</html>
```

> **Warning**
>
> The files at the root of `files.stork-search.net` point directly to the build artifacts from most recent release. Linking to these files from your webpage can result in unexpected behavior when a new version is released. To pin to a specific release, use URLs in the following the format:
>
> `https://files.stork-search.net/releases/v2.0.0/stork.js`
>
> → [Read more about the `files.stork-search.net` fileserver]()

## Going further

With Stork, you can:

- [Create a search interface for videos or podcasts, and link directly to specific timestamps]()
- [Automatically create and publish a search index when you update your blog]()
- [Index content in multiple languages]()
- [Self-host the WASM, JS, CSS, and search indexes to manage your own infrastructure]()
- ...and more!

## Contributing

Stork gratefully accepts contributions!

- If you find a bug, a well-written bug report is extremely helpful. [File a bug here](https://github.com/jameslittle230/stork/issues/new?assignees=&labels=bug&template=bug_report.md&title=).
- If you want to ask about a potential feature request or enhancement, feel free to [start a discussion](https://github.com/jameslittle230/stork/discussions) or [send a message on Discord](https://stork-search.net/chat).
- If you want to write some code, the [Contributing Guidelines](https://github.com/jameslittle230/stork/blob/master/docs/contributing.md) can help make sense of how to get started.
- I'd appreciate a heads up before you submit a PR so I can make sure that nobody is duplicating work.

In order to make sure that the Stork community is welcoming to all, please review and follow the guidelines laid out in the [Stork Code of Conduct](https://github.com/jameslittle230/stork/blob/master/.github/CODE_OF_CONDUCT.md).
