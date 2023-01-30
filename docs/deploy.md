# Deploying Stork

<!-- just print-deploy -->

## Preflight

- [ ] Check out master locally
- [ ] Run `just dev` to begin dev site
- [ ] Check that Stork works in the dev site
- [ ] Run `just generate-stats` to ensure the script still works
- [ ] Begin an Ubuntu 22.04 t2.medium EC2 instance
- [ ] Use Bob to fetch the master branch
- [ ] Run `just generate-stats`
- [ ] Ensure values are acceptable, compared to existing version.
  - If not, abort the release and debug.
- [ ] Copy benchmark values into notes
- [ ] Write a changelog
- [ ] Create a new branch locally. In that branch:
  - [ ] Run `just set-version A.B.C`
  - [ ] Update the dependency on lib in the wasm and cli crates
  - [ ] Commit, push branch, and open a PR
- [ ] While release PR is building, create a local site branch. In that branch:
  - [ ] Add benchmark values
  - [ ] Update all CDN references to the updated version number
  - [ ] Add documentation, if applicable
  - [ ] Commit, push branch, and open a PR
  - [ ] Let the Netlify preview build in the background

## Release

- [ ] Merge release PR
- [ ] Run `git fetch origin`
- [ ] Check out and pull latest master branch
- [ ] Run `just tag-version A.B.C`. This pushes the new tag.
- [ ] Wait for the release to be built. Github Actions will deploy the release automatically.

## Afterwards

- [ ] Check that the demo the site's Netlify preview works.
  - If not, abort the release and debug.
- [ ] Add the changelog from to the Github release, and publish the release
- [ ] Merge the PR you made on [the documentation site](https://github.com/stork-search/site)
- [ ] Create an Amazon Linux binary.
  - [ ] Begin an Amazon Linux t2.medium EC2 instance
  - [ ] Use Bob to build the binary
  - [ ] Upload that binary to Github Release
  - [ ] Upload that binary to S3
- [ ] Create an ARM macOS binary
  - [ ] Use Bob on my local computer to build a binary
  - [ ] Upload that binary to Github Release
  - [ ] Upload that binary to S3
- [ ] Update Homebrew
  - [ ] Generate a new brewfile based on the Github-generated tarball:
    - [ ] `$ rm /opt/homebrew/Library/Taps/homebrew/homebrew-core/Formula/stork.rb` on my computer
    - [ ] `$ brew create https://github.com/jameslittle230/stork/archive/vX.Y.Z.tar.gz`
  - [ ] Manually update the URL and SHA in the [Homebrew formula file](https://github.com/jameslittle230/homebrew-stork-tap/blob/master/Formula/stork.rb)
- [ ] Run `$ cargo publish` from within master branch
