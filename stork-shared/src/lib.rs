use std::collections::HashMap;

pub trait StorkIndex {}

pub type Fields = HashMap<String, String>;

mod stopwords;
pub use stopwords::STOPWORDS as stopwords;
