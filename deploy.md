# Deploying Stork

## Prepare

- [ ] Start the AWS benchmark and build instances
- [ ] Run `just generate-stats` on my computer to make sure it works
- [ ] Run benchmarks from benchmarking machine
  - [ ] Log into benchmarking machine
  - [ ] Update to master
  - [ ] Run `just generate-stats`
  - [ ] Ensure values are acceptable, compared to existing version. If not, abort the release and debug.
  - [ ] If so, copy benchmark values into notes (you'll need them later!)
- [ ] Create a release PR:
  - [ ] Add date to changelog
  - [ ] Bump versions
    - [ ] In package.json
    - [ ] In Cargo.toml for the lib, wasm, and cli crates
    - [ ] The dependency on lib in the wasm and cli crates
  - [ ] Commit to master and push
- [ ] Create a new PR on the site
  - [ ] Add benchmark values
  - [ ] Update all CDN references to the updated version number
  - [ ] Add documentation, if applicable
  - [ ] Let the Netlify preview build in the background

## Release

- [ ] On your computer, check out the latest master
- [ ] Run `$ git tag -a vX.Y.Z -m "Release version X.Y.Z"`
- [ ] Run `$ git push origin vX.Y.Z`
- [ ] Wait for the release to be built. Github Actions will deploy the release automatically.

## Aftercare

- [ ] Check that the demo the site's Netlify preview works. If not, abort the release and debug.
- [ ] Add the changelog to the Github release, and publish it
- [ ] Create an Amazon Linux binary.
  - [ ] Run the shell script from the amazon-linux build machine
  - [ ] Upload the Amazon Linux binary to the CDN
  - [ ] Upload the Amazon Linux binary to the Github release.
- [ ] Merge the PR you made on [the documentation site](https://github.com/stork-search/site)
- [ ] Update Homebrew
  - [ ] Generate a new brewfile based on the Github-generated tarball:
    - [ ] `$ rm /opt/homebrew/Library/Taps/homebrew/homebrew-core/Formula/stork.rb` on my computer
    - [ ] `$ brew create https://github.com/jameslittle230/stork/archive/vX.Y.Z.tar.gz`
  - [ ] Manually update the URL and SHA in the [Homebrew formula file](https://github.com/jameslittle230/homebrew-stork-tap/blob/master/Formula/stork.rb)
- [ ] Run `$ cargo publish`
