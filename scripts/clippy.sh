#!/bin/bash
cargo clean
cargo clippy -- --W clippy::pedantic \
-A clippy::upper_case_acronyms \
-A clippy::missing_panics_doc \
-A clippy::missing_errors_doc \
-A clippy::too_many_lines \
-A clippy::must_use_candidate \
-A clippy::module_name_repetitions