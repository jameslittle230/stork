pub enum ProgressReport {
    ProgressTick(TickModel),
    Error(String),
}

pub struct TickModel {
    pub total_document_count: usize,
    pub state: TickState,
}

pub enum TickState {
    StartedDocument { index: usize, title: String },
    Finished,
    Failed,
}
