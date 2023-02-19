pub trait ProgressReporter {
    fn send_warning(&self, str: String);
    fn fail(&self);
    fn succeed(&self);
    fn send_tick(&self, index: usize, total: usize, display: String);
}
