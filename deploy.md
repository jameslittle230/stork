# Deploying Stork

## Prepare

- [ ] Log into the benchmarking machine
  - [ ] Update to master
  - [ ] Run benchmarks
  - [ ] Ensure values are acceptable. If not, abort the release and debug.
- [ ] Create a release PR:
  - [ ] Add date to changelog
  - [ ] Bump the version in package.json and in Cargo.toml for the lib, wasm, and cli crates
  - [ ] Commit to master and push
- [ ] Create a new PR on the site with new documentation, if applicable

## Release

- [ ] On your computer, check out the latest master
- [ ] Run `$ git tag -a vX.Y.Z -m "Release version X.Y.Z"`
- [ ] Run `$ git push origin vX.Y.Z`
- [ ] Wait for the release to be built. Github Actions will deploy the release automatically.

## Aftercare

- [ ] Check that the demo on <https://stork-search.net> and elsewhere still works.[^ifnot]
- [ ] Add the changelog to the Github release, and publish it
- [ ] Create an Amazon Linux binary and upload it to the CDN and release
- [ ] Merge the PR you made on [the documentation site](https://github.com/stork-search/site)
- [ ] Update Homebrew
  - [ ] Generate a new brewfile based on the Github-generated tarball:
    - [ ] `$ rm /usr/local/Homebrew/Library/Taps/homebrew/homebrew-core/Formula/stork.rb`
    - [ ] `$ brew create https://github.com/jameslittle230/stork/archive/vX.Y.Z.tar.gz`
  - [ ] Manually update the URL and SHA in the [Homebrew formula file](https://github.com/jameslittle230/homebrew-stork-tap/blob/master/Formula/stork.rb)

1. Run `$ cargo publish`
1. Download the build artifacts from Github and upload them to S3

[^ifnot]: If the demo on the site doesn't work, or if other Stork integrations on the web don't work, take the versioned build artifacts from S3 and replace the root files with those artifacts, then run another Cloudfront cache invalidation from the AWS UI.
