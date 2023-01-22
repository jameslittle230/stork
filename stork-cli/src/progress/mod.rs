use std::cell::RefCell;

use indicatif::{ProgressBar, ProgressStyle};
use stork_lib as lib;
use strunc::Strunc;

/// An API surrounding Indicatif's `ProgressBar`, giving the bar an opinionated
/// style & behavior, and setting progress based on
/// `lib::build_output::progress::Report` values.
///
/// ```
/// let bar = progress::Bar::new();
/// for i in 0..10 {
///     let doc_name = format!("Document {}", i + 1)
///     bar.tick(progress::report {
///         total_document_count: (0..10).count(),
///         state: lib::build_output::ProgressState::StartedDocument {
///             index: i,
///             title: doc_name,
///         },
///     });
///
///     std::thread::sleep_ms(1000);
/// }
///
/// bar.tick(progress::report {
///     total_document_count: (0..10).count,
///     state: lib::build_output::ProgressState::Failed,
/// });
/// ```
pub(crate) struct Bar {
    progress_bar: ProgressBar,
    internals: RefCell<BarInternalInfo>,
}

struct BarInternalInfo {
    first_tick: bool,
    length: u64,
}

impl Bar {
    pub(crate) fn new() -> Self {
        let progress_bar = ProgressBar::new(100).with_style(
            ProgressStyle::default_bar()
                .template("[{elapsed}] {bar:40.cyan/blue} {pos}/{len} | {spinner} {msg}")
                .unwrap()
                .progress_chars("##-")
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏✓"),
        );

        Self {
            progress_bar,
            internals: RefCell::new(BarInternalInfo {
                first_tick: false,
                length: 0,
            }),
        }
    }

    pub(crate) fn tick(&self, report: lib::build_output::ProgressReport) {
        if !self.internals.borrow().first_tick {
            self.progress_bar
                .enable_steady_tick(std::time::Duration::from_millis(100));
            self.internals.borrow_mut().first_tick = true;
        }

        if report.total_document_count as u64 != self.internals.borrow().length {
            self.progress_bar
                .set_length(report.total_document_count as u64);
            self.internals.borrow_mut().length = report.total_document_count as u64;
        }

        match report.state {
            lib::build_output::ProgressState::StartedDocument { index, title } => {
                self.progress_bar.set_position(index as u64);
                self.progress_bar.set_message(title.strunc().to_string());
            }
            lib::build_output::ProgressState::Finished => self
                .progress_bar
                .finish_with_message(colored::Colorize::green("Done :)").to_string()),
            lib::build_output::ProgressState::Failed => self
                .progress_bar
                .abandon_with_message(colored::Colorize::red("Failed :(").to_string()),
        }
    }
}
