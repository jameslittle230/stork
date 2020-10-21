// pub mod v1; // RIP
pub mod v2;
pub mod v3;

use crate::searcher::index_analyzer::VersionParseError;
use crate::common::IndexFromFile;
use wasm_bindgen::JsValue;

pub enum ParsedIndex {
   V2(v2::structs::Index),
   V3(v3::structs::Index),
}

#[derive(Debug)]
pub enum IndexParseError {
   VersionParseError(VersionParseError),
   DecodeError(rmp_serde::decode::Error),
}

impl From<VersionParseError> for IndexParseError {
   fn from(e: VersionParseError) -> IndexParseError {
      IndexParseError::VersionParseError(e)
   }
}

impl From<rmp_serde::decode::Error> for IndexParseError {
   fn from(e: rmp_serde::decode::Error) -> IndexParseError {
      IndexParseError::DecodeError(e)
   }
}

impl std::fmt::Display for IndexParseError {
   fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
      match self {
         IndexParseError::DecodeError(e) => write!(f, "Error decoding index: {}", e),
         IndexParseError::VersionParseError(e) => write!(f, "Error parsing index version: {}", e),
      }
  }
}

impl Into<JsValue> for IndexParseError {
   fn into(self) -> JsValue {
      js_sys::Error::new(&self.to_string()).into()
  }
}

impl std::convert::TryFrom<&IndexFromFile> for ParsedIndex {
   type Error = IndexParseError;

   fn try_from(index: &IndexFromFile) -> Result<ParsedIndex, IndexParseError> {
      use crate::index_versions;
      use crate::searcher::index_analyzer::{parse_index_version, IndexVersion};

      let parsed_index = match parse_index_version(index)? {
          IndexVersion::V2 => {
              let index = index_versions::v2::structs::Index::from_file(index);
              ParsedIndex::V2(index)
          }
          IndexVersion::V3 => {
              let index = index_versions::v3::structs::Index::try_from(index)?;
              ParsedIndex::V3(index)
          }
      };

      Ok(parsed_index)
   }
}
