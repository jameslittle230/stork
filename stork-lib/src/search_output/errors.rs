use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("")]
    NotCompiledWithFeature,

    #[error("")]
    MethodNotAvailableForIndex,
}
