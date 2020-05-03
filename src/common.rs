use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type Fields = HashMap<String, String>;
pub type IndexFromFile = [u8];

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum InternalWordAnnotation {
    #[serde(rename = "a")]
    SRTUrlSuffix(String),
}
