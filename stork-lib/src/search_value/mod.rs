use crate::index_v4::QueryResult;

pub(crate) struct V4SearchValue {
    pub(crate) result: QueryResult,
    pub(crate) chars_remaining: u8,
}

/// An opaque struct that contains data about a search result. These values can
/// be cached piecewise by `SearchTerm`, and can be resolved later.
pub struct SearchValue {
    pub(crate) v4_value: Option<V4SearchValue>,
}
