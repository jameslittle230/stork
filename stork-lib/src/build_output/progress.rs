/// If you're displaying a progress bar while building a search index, this data
/// helps build that progress bar.
pub struct Report {
    pub total_document_count: usize,
    pub state: State,
}

pub enum State {
    StartedDocument { index: usize, title: String },
    Finished,
    Failed,
}
