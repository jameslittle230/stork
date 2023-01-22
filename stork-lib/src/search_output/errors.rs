#[derive(Debug)]
pub enum SearchError {
    NotCompiledWithFeature,
    MethodNotAvailableForIndex,
}
