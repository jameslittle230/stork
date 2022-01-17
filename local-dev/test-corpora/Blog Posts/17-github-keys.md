---
title:  "The Most Efficient Github SSH Key Generation Process"
date:   2018-03-20
layout: post
tags: post
blurb: "I generate SSH keys a lot. Here's how to make them (and add them to your Github account) as fast as humanly possible."
---
# 17-github-keys

I generate SSH keys a lot. Here's how to make them (and add them to your Github account) as fast as humanly possible.

1. Open <https://github.com/settings/ssh/new> in a new tab
2. Run `ssh-keygen -t rsa -b 4096` in your terminal. Make a passphrase. Save the key in the default location.
3. Run `eval "$(ssh-agent -s)"; ssh-add ~/.ssh/id_rsa; cat ~/.ssh/id_rsa.pub`
4. Copy and paste the terminal output into the Github page you opened before.

Title format courtesy of [Hammacher Schlemmer](https://www.hammacher.com/product/most-efficient-fireplace-grate).