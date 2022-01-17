---
title:  "Stork and Modularity"
date: 2021-09-19
layout: post
tags: post
blurb: "Splitting code into crates, the benefits of small public interface, and a combinatorial explosion."
---

I've been working on making [Stork](https://stork-search.net) a more modular project. Stork has a lot of code in it; to make it more readable, I've been putting that code in different files. With Rust, every file is a different module (for the most part), so this winds up with lots of stacked modules. Rust also lets you only reference modules absolutely (starting from the crate) or relatively (by ascending up the module stack and then descending down), so my imports were getting kind of unwieldy. 

```rust
super::super::super::config::file::SRTSubtitleFormat // :(
```

This has been mostly manageable, but recently, [Andrew](https://healeycodes.com) wanted to compile Stork on an environment (x86 Amazon Linux 2) for which my release process doesn't create a binary. I spun up an ec2 instance, tried to compile Stork, and was disappointed to find that it doesn't compile - the new "download a web page and index it" feature relies on OpenSSL, which the Rust installation was having trouble finding.[^0] As a workaround, I wanted to see if there was a way to conditionally compile out that code. Andrew wouldn't be able to download web pages and index them, but that's better than not being able to run Stork after all.

In a Rust crate, you can use [features](https://doc.rust-lang.org/cargo/reference/features.html) to mark code as optional. When the feature is on, it will be compiled into the binary. When the feature is off, the compiler will act as if that code isn't there. You can also mark crates as installed only when a feature is enabled, so my original plan was to put that web-page-downloading feature (and the crates it depended on) behind a feature and try compiling Stork with that feature disabled.

While I was poking at my features, I wanted to change how I approached the modularity of the project. There are some aspects of the code, like a "common" module which contains things like constants, type aliases, and some shared models, that I wanted to extract into its own crate. In fact, there are several modules in the root of Stork that could be their own crate. For example, Stork supports parsing and searching indexes from old versions of Stork, which means the WASM binary has to include a separate deserialization schema that 99%[^1] of people don't use. It'd be nice to offer a smaller WASM binary that doesn't include that code, if folks know they won't need it. So this weekend, I got up close and personal with my Stork repo to split out my WASM bridge models, my v2 index, my v3 index, and my config models into their own crates.

As usual, the compiler admonished me repeatedly.[^2] I was referencing things across modules in a way that wasn't allowed -- or at least, that I shouldn't repeat -- across crates. I updated nearly every import in the project. It felt like I was finally making some of the sketchier parts of the codebase clearer, which felt nice. Along the way, I cleaned up my custom error implementations with the [`thiserror`](https://github.com/dtolnay/thiserror) crate, which was really satisfying. And getting to a place where I had easily understandable crates with small public interfaces all interacting with each other made me look forward to building on top of those crates in the future.[^3]

And yet, this project seemed futile for a good portion of it. Stork still doesn't compile, so there's more work to do. None of the work seemed all that meaningful -- I wasn't adding new features, I was hacking on my import statements. Most of all, though, was the combinatorial explosion of compilable versions of Stork. Now, in my CI, I'll have to test 5 crates' test suites with different sets of features enabled and disabled, and I believe I'll have to define all those combinations manually. My local development (rust-analyzer) seems to have broken itself after looking at code that doesn't exist in the compiled output for so long. I've added more boilerplate in the form of crate definitions. I'm worried that I've made my life more complex to make the public interfaces of my code less so. I can see the benefit of enforcing better public interfaces immediately, but I won't really feel the pain of the combinatorial explosion until later, so right now, I'm biased towards thinking this was a good idea when it might actually be a bad one that will only sneak up on me after a few months.

I want to keep hacking on this, mostly because I'm _so close_ and because it'd be nice to be able to give Andrew a working binary. But I think I realized that spending a weekend on my build configurations doesn't spark as much joy as building something new.

[^0]: This seems like a bug? I'll have to go back and reproduce it and file it later, probably on [this repo](https://github.com/sfackler/rust-openssl).

[^1]: Or 100%! I don't have the analytics to determine, though none of the indexes I've seen through the analytics I _do_have are using the old format.

[^2]: This isn't a bad thing! Compiler admonition is much better than runtime failure; that's one of the reasons I like writing Rust so much.

[^3]: In Rust, "Crate" is its own visibility level. Now that there are more crate boundaries in the project, I can be a lot more granular about what symbols are visible, so I can make more symbols private (or make symbols private more easily) than I could before. That makes me more likely to write documentation for my public interfaces (or even care about them when I previously hadn't), since those public interfaces are now more logical and meaningful than they were before.