#!/bin/bash

# If you change this, also change it in ci-on-push.yml
cargo clippy --fix --allow-dirty -- --W clippy::pedantic \
-A clippy::upper_case_acronyms \
-A clippy::missing_panics_doc \
-A clippy::missing_errors_doc \
-A clippy::too_many_lines \
-A clippy::must_use_candidate \
-A clippy::module_name_repetitions \
-A clippy::semicolon_if_nothing_returned \
-A clippy::default_trait_access \
-A clippy::field_reassign_with_default \