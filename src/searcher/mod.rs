use crate::IndexFromFile;
use serde::Serialize;

#[derive(Serialize)]
pub struct SearchResults {

}

pub fn search(_index: &IndexFromFile, _query: &str) -> SearchResults {
    SearchResults{}
}