use crate::{build_config, build_output};

pub(crate) struct ProgressReporter {}

impl ProgressReporter {
    pub(crate) fn new(
        config: &build_config::Config,
        progress_fn: Option<&dyn Fn(build_output::ProgressReport)>,
    ) -> Self {
        Self {}
    }

    pub(crate) fn report(&self, state: build_output::ProgressState) {}

    fn should_report_progress(config: &build_config::Config) -> bool {
        config.input.files.len() > 1000
            || config
                .input
                .files
                .iter()
                .any(|file| matches!(file.source(), crate::build_config::DataSource::URL(_)))
    }
}
