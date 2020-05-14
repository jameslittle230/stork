# How to Deploy a New Stork Version

## Prepare
1. Bump the version in package.json and Cargo.toml
1. Commit the version bump to master and push
1. Make sure `master` passes
1. Prepare documentation and stats on stork-site

## Build
1. Run `$ yarn build:prod`
1. Create a new draft Github release. Set the tag and the version name to be the new version number: `v0.7.0`.

## Deploy
1. Run `$ ./scripts/upload_federalist_exe.sh`
1. Check that the demo on https://stork-search.net still works. If so:
1. Push the draft Github release
1. Push the new content to stork-site
1. Generate a new brewfile based on the Github-generated tarball: `$ brew create https://github.com/jameslittle230/stork/archive/v0.6.0.tar.gz`
1. Manually update the URL and SHA in the [Homebrew formula file](https://github.com/jameslittle230/homebrew-stork-tap/blob/master/Formula/stork.rb)
1. run `$ cargo publish`
