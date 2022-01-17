---
title:  "Something I wish CSS Grid did, but it doesn't"
date: 2020-02-01
layout: post
tags: post
blurb: "Can I, just, like, complain for a second? Thanks."
---

I redesigned my website! It uses Eleventy now and some bits should be better but a lot of the bits remain the same.

I cared a lot about the page layout for this version—the layout of the last redesign was kind of awful. I wanted to align things on a grid, and I knew that other people had found that CSS Grid was becoming a better and better layout tool, so I wanted to see if I could put my entire website in a grid and make the layout fall into different grid areas.

I knew I wanted a central column for text, and I wanted that central column to be nicely sized so your eyes don't get tired tracking across super-long lines of text. I also wanted some elements to be able to break out of that central column into a wider, but still centered column. You can likely see the intended result on the [portfolio page](/portfolio), but here's a diagram:

<img src="https://files.jameslittle.me/images/layout-2.png" alt="All elements fit into a central column, except for one which is wider." />

With an indended HTML structure of:

```html
<div class="grid">
  <header>...</header>
  <main>
    <p>...</p>
    <p>...</p>
    <div class="breakout">...</div>
    <p>...</p>
  </main>
</div>
```

Turns out this is impossible, as far as I can tell. Only direct child elements are grid sub-items, and if I have (as displayed here) a `<div>` inside a `<main>` inside a `display: grid` element, the `<main>` will be the grid item (and will be able to conform itself to a given grid area), but the contents inside don't have access to that grid.

If I change the `<main>` to be the grid, then every element inside it will try to fit inside a whole grid area; there's no concept of laying out elements in a normal page flow if they're all in the same grid area and then modifying one element to exist in a superset of that area.

I hacked around it with a carefully calculated negative left margin, and I get why CSS grid works the way it does (in essence, it's not the right tool for the job I want to do), but there still seems to be room for CSS layout to improve here. I've watched CSS authors get less reliant on hacks to encode certain types of layouts, but I'm still not sure we've found the right abstraction for someone who wants to make a layout like mine.
