# Changelog

## v1.4.0

[DATE TBD](https://github.com/jameslittle230/stork/releases/tag/v1.4.0)

### New Features

- Configuration files can now be in JSON format, in addition to TOML format

### Bug Fixes

- Removes a stray `console.log` from the Javascript application
- Fixes a Javascript runtime bug where registered indexes weren't always reporting as ready

## v1.3.0

[Nov 23, 2021](https://github.com/jameslittle230/stork/releases/tag/v1.3.0)

### New Features

- Indexes `alt` and `title` attributes on HTML elements
- Adds configuration keys to set an HTML selector as excluded from indexing
- Adds a configuration key, `output.save_nearest_html_id`, that, when set to true, will index the nearest HTML IDs for each word. The web interface will link to that ID; clicking on that search result will jump to the text's location on the page.
- Reduces JS and WASM artifact sizes by changing build system settings
- Updates CSS themes, and adds a new theme: `flat`

### Bug Fixes

- Fixes a bug where users were able to accidentally download two instances of Stork's WASM on the page (Thanks [@justinmayer](https://github.com/justinmayer)!)
- Fixes a bug where the indexer was hanging in environments where stdin was not passed in as an empty stream (Thanks [@Aethon](https://github.com/Aethon)!)

### Other

- Updates dependencies

## v1.2.1

[May 11, 2021](https://github.com/jameslittle230/stork/releases/tag/v1.2.1)

**Bug fixes:**

- Fixes issue where \[x\] button in Basic or Dark themes wouldn't respond to the resizing of the `stork-wrapper` container ([#176](https://github.com/jameslittle230/stork/pull/176))
- Fixes issue where searching for three characters wouldn't display results on the web page ([#172](https://github.com/jameslittle230/stork/pull/172))
- Fixes crash when the title of a document included non-unicode characters ([#173](https://github.com/jameslittle230/stork/pull/173) - thanks [@Erwan-le-Gall](https://github.com/Erwan-le-Gall)!)

## v1.2.0

[Apr 21, 2021](https://github.com/jameslittle230/stork/releases/tag/v1.2.0)

### New Features

- Stork can now index content from the web. (When the docs are available, a link to the docs will be here!) [#146](https://github.com/jameslittle230/stork/pull/146)
- Stork's command line interface has been redesigned and rewritten, with backwards-compatible shims added where needed. [#160](https://github.com/jameslittle230/stork/pull/160)
  - This change deprecates the `filename` key in the output configuration.

### Quality of Life Improvements

- If you index a file and get an empty buffer, Stork will let you know there might be a problem. [#147](https://github.com/jameslittle230/stork/issues/147)
- Adds debug method to JS interface [#161](https://github.com/jameslittle230/stork/pull/161)
- Improves command line output, especially for errors [#160](https://github.com/jameslittle230/stork/pull/160)
- Adds a new `break_on_file_error` configuration option to stop indexing when first file fails, rather than continuing without the erroring file. [#160](https://github.com/jameslittle230/stork/pull/160)

### Bug Fixes

- Use semantic HTML for search highlight [#155](https://github.com/jameslittle230/stork/pull/155) (Thanks, [bronzehedwick](https://github.com/bronzehedwick)!)
- Fix JS idempotency bugs [#153](https://github.com/jameslittle230/stork/pull/153)

## v1.1.0

[Feb 15, 2021](https://github.com/jameslittle230/stork/releases/tag/v1.1.0)

**New Features**

- Added self-hosting support. Read the [self-hosting documentation](https://stork-search.net/docs/self-hosting) to learn more.
- Added Javascript lifecycle methods to give you control over when the WASM downloads, when the index file is downloaded, and when Stork attaches to the DOM. This will greatly improve the Stork experience when using React-based static site generators, such as Next.js or Gatsby. Read the [Advanced JS documentation](https://stork-search.net/docs/advanced-js) to learn more.
- New Javascript API method for searching an index without requiring that you use Stork's UI. If you want to build your own Stork UI from scratch, this is the method for you. The [Advanced JS documentation](https://stork-search.net/docs/advanced-js) link will help you get started with this, too.
- New Javascript configurations:
  - `onResultsHidden` - Callback that gets called when the results are hidden, when the user presses esc or clicks on the close button
  - `onInputCleared` - Callback that gets called when the input is cleared, when the user presses esc twice
  - `showCloseButton` - Boolean to determine whether the close button is visible or not
- Stork can now take in a configuration file that's piped into the `$ stork --build` command, instead of requiring that you pass in a file path.

## v1.0.4

[Jan 10, 2021](https://github.com/jameslittle230/stork/releases/tag/v1.0.4)

**Bug Fixes:**

- The `html_selector` option in the configuration file wasn't being parsed correctly, leading to the feature seemingly not working

**Enhancements:**

- Better error message when there are no valid files

## v1.0.3

[Jan 2, 2021](https://github.com/jameslittle230/stork/releases/tag/v1.0.3)

**Bug Fixes:**

- Javascript library was erroring incorrectly if the output HTML element could not be found
- Javascript library was adding stray elements to the DOM while the index was loading

**Enhancements:**

- Hyphens are now treated the same as spaces for indexing and searching purposes. In effect, you can now search for `avon` and it will match the term `Stratford-upon-Avon` in your index.
- Stork used to fail the entire indexing process if there was an error parsing a single file. Now, it will collect those errors and present them in the console, but still build an entire index with the remaining files.

> Note: Javascript bug fixes get applied automatically if you're loading the Stork library from `files.stork-search.net`.

## v1.0.2

[Dec 29, 2020](https://github.com/jameslittle230/stork/releases/tag/v1.0.2)

**Bug Fixes:**

- Fix highlight offsets when excerpt contains multi-byte characters. Requires that you re-build your index. (Thanks for reporting, [@DanilaFe](https://github.com/DanilaFe)!)

**Error Improvements:**

- Describe which HTML selector cannot be found when an HTML document fails to index.
- Collect and display indexing errors instead of failing early (Thanks for suggesting, [@fauno](https://github.com/fauno)!)

## v1.0.1

[Dec 28, 2020](https://github.com/jameslittle230/stork/releases/tag/v1.0.1)

**Bug fixes:**

- Some browsers wouldn't display results properly if you started typing before the WASM file had loaded (thanks, [@reese](https://github.com/reese)!)
- The indexer wouldn't index words past a certain point if the document contained non-word space-delimited tokens. For example, if your document had the contents `hello - world`, the word `world` wouldn't be indexed. (Thanks for reporting, [@DanilaFe](https://github.com/DanilaFe)!)
- Fix a webpack bug that was only encountered on first install

Thanks for using Stork!

## v1.0.0

[Dec 13, 2020](https://github.com/jameslittle230/stork/releases/tag/v1.0.0)

Stork 1.0.0 is here, with new features, stability improvements, and lots more speed. The major version bump signifies that Stork is officially out of beta, and that I (James, the developer) believe that it can be "production ready" on your site.

As with any Stork version bump, your Javascript library will automatically update and get you some of the latest changes, and will still work with the index you've already built. To get the full benefit of all the changes in 1.0.0, make sure to update the version of Stork that you're running, then rebuild your index.

**Index Generator Updates:**

- Stork can parse HTML and Markdown input files
- `stork --test` command will open a local webserver that lets you test a generated index, without having to build it
- Sped up index generation (Thanks [@DenialAdams](https://github.com/DenialAdams)!)

**Javascript Library Updates:**

- Separated index parsing from searching in generated WASM library, which sped up searches (Thanks [@DenialAdams](https://github.com/DenialAdams)!)
- Javascript library adds a "Powered by Stork" UI
- Adds event handlers to the Javascript configuration so you can define callbacks when your users search for queries and click on results

**Other Improvements:**

- Reworked public interface for those using Stork as a Rust library
- Added dark.css theme
- New website documentation
