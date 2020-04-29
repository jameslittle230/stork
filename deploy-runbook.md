# How to Deploy a New Stork Version

## Prepare
1. Bump the version in package.json and Cargo.toml
1. Commit the version bump to master and push
1. Make sure `master` passes
1. Prepare documentation, changelog, stats and roadmap on stork-site

## Build
1. Run `$ yarn build:prod`
1. Create a new draft Github release with a changelog

## Deploy
1. Run `$ ./scripts/upload_federalist_exe.sh`
1. Push the draft Github release
1. Push the new content to stork-site
1. Generate a new brewfile: `$ brew create https://github.com/jameslittle230/stork/archive/v0.6.0.tar.gz`
1. Update the URL and SHA in the [Homebrew formula file](https://github.com/jameslittle230/homebrew-stork-tap/blob/master/Formula/stork.rb)
