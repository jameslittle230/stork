use std::{cell::RefCell, time::Instant};

use colored::Colorize;
use indicatif::{ProgressBar, ProgressStyle};
use lib::{
    build::ProgressReporter,
    build_config::{Config, DataSource},
};
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
    creation_time: Instant,
    config_contains_urls: bool,
    first_tick: bool,
    length: usize,
}

impl Bar {
    pub(crate) fn new(config: &Config) -> Self {
        let progress_bar = ProgressBar::new(100).with_style(
            ProgressStyle::default_bar()
                .template("[{elapsed}] {bar:40.cyan/blue} {pos}/{len} | {spinner} {msg}")
                .unwrap()
                .progress_chars("##-")
                .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏✓"),
        );

        let config_contains_urls = config
            .input
            .files
            .iter()
            .any(|file_config| matches!(file_config.source(), Ok(DataSource::URL(_))));

        Self {
            progress_bar,
            internals: RefCell::new(BarInternalInfo {
                creation_time: Instant::now(),
                config_contains_urls,
                first_tick: false,
                length: 0,
            }),
        }
    }

    fn is_visible(&self) -> bool {
        let taken_longer_than_2s = Instant::now()
            .duration_since(self.internals.borrow().creation_time)
            .as_secs()
            >= 2;

        taken_longer_than_2s || self.internals.borrow().config_contains_urls
    }
}

impl ProgressReporter for Bar {
    fn send_warning(&self, str: String) {
        self.progress_bar.suspend(|| {
            println!("{} {str}", "Warning:".yellow());
        });
    }

    fn succeed(&self) {
        if !self.is_visible() {
            return;
        }

        self.progress_bar
            .finish_with_message(colored::Colorize::green("Done :)").to_string());
    }

    fn fail(&self) {
        if !self.is_visible() {
            return;
        }

        self.progress_bar
            .finish_with_message(colored::Colorize::red("Failed :(").to_string());
    }

    fn send_tick(&self, index: usize, total: usize, display: String) {
        if !self.is_visible() {
            return;
        }

        if !self.internals.borrow().first_tick {
            self.progress_bar
                .enable_steady_tick(std::time::Duration::from_millis(100));
            self.internals.borrow_mut().first_tick = true;
        }

        if total != self.internals.borrow().length {
            self.progress_bar.set_length(total as u64);
            self.internals.borrow_mut().length = total;
        }

        self.progress_bar.set_position(index as u64);
        self.progress_bar.set_message(display.strunc().to_string());
    }
}
