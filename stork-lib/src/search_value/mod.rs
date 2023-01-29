use crate::{index_v4::SearchValue as V4SearchValue, search_query::SearchTerm};

/// An opaque struct that contains data about a search result. These values can
/// be cached piecewise by `SearchTerm`, and can be merged later.
#[derive(Clone)]
pub struct SearchValue {
    pub(crate) v4_value: Option<V4SearchValue>,
}

#[derive(Clone)]
pub struct SearchValueCacheKey {
    index_name: String,
    search_term: SearchTerm,
}
