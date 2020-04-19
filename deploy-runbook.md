# How to Deploy a New Stork Version

1. Make sure `master` passes
1. Bump the version in package.json and Cargo.toml
1. Run `$ cargo build`
1. Commit the version bump to master and push
1. Create a new Github release with a changelog
1. Copy the built binary and rename to `stork-0.5.0-macos`
1. Upload the binary to the Github release
1. Run `$ ./scripts/upload_federalist_exe.sh`
1. If needed, update documentation, changelog, and roadmap on stork-site
1. Update the stats on stork-site
1. Update the exe that homebrew links to: <https://github.com/jameslittle230/homebrew-stork-tap>