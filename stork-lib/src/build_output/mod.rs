//! The data that you might receive during and after the index build process.
//!
//! During the build, you may get progress reports with the `ProgressReport` struct,
//! which reports a `ProgressState`. You can use these values to render a progress bar.
//!
//! After a successful build, you will get a `BuildSuccessValue` which contains
//! data that you can use to generate an index file, as well as give success output
//! to the user.
//!
//! Along with a successful build, you might get a set of `BuildWarning`s reported
//! simultaneously. These are non-fatal errors that took place while building the index,
//! and receiving `BuildWarning`s may mean that the search index is well-formed, but
//! incomplete. To require that an index be built with no warnings, you can include
//! the `break_on_file_error` field in the `[input]` section of your configuration file.
//!
//! A failing build will fail with one of the errors described in the `errors` module.

mod progress;
mod success;
mod warning;

pub(crate) mod document_problem;

pub mod errors;

pub use progress::{Report as ProgressReport, State as ProgressState};
pub use success::{BuildStatistics, Value as BuildSuccessValue};
pub use warning::BuildWarning;
