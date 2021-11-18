use crate::index_versions::{v2, v3};
use crate::IndexFromFile;
use serde::Serialize;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum IndexVersion {
    V2,
    V3,
}

impl fmt::Display for IndexVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", {
            match self {
                IndexVersion::V2 => v2::VERSION_STRING,
                IndexVersion::V3 => v3::VERSION_STRING,
            }
        })
    }
}

impl std::convert::From<ParsedIndex> for IndexVersion {
    fn from(index: ParsedIndex) -> Self {
        match index {
            ParsedIndex::V2(_) => Self::V2,
            ParsedIndex::V3(_) => Self::V3,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum VersionSizeProblem {
    Short,
    Long,
}

impl fmt::Display for VersionSizeProblem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", {
            match self {
                VersionSizeProblem::Short => "short".to_string(),
                VersionSizeProblem::Long => "long".to_string(),
            }
        })
    }
}

#[derive(Debug, PartialEq)]
pub enum IndexParseError {
    FileTooShort,
    BadVersionSize(u64, VersionSizeProblem),
    VersionStringUtf8Error(std::string::FromUtf8Error),
    UnknownVersionString(String),

    // This could have an rmp::decode associated value, but I'm opting to
    // suppress it.
    DecodeError(String),
}

impl Error for IndexParseError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        if let IndexParseError::VersionStringUtf8Error(from_utf8_error) = self {
            Some(from_utf8_error)
        } else {
            None
        }
    }
}

impl From<rmp_serde::decode::Error> for IndexParseError {
    fn from(e: rmp_serde::decode::Error) -> IndexParseError {
        IndexParseError::DecodeError(e.to_string())
    }
}

impl fmt::Display for IndexParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            IndexParseError::FileTooShort => {
                "Index file too short; could not find a version string.".to_string()
            }

            IndexParseError::BadVersionSize(size, problem) => format!(
                "Version size `{}` is too {}; this isn't a valid index file.",
                size,
                problem
            ),

            IndexParseError::VersionStringUtf8Error(e) => format!(
                "Could not parse version string as valid UTF8, got:\n{:?}",
                e.as_bytes()
            ),

            IndexParseError::UnknownVersionString(string) => {
                format!("Unknown index version `{}` found", string)
            },

            IndexParseError::DecodeError(error) => format!(
                "Could not decode index, bruh! (internal error {}). If you see this, please file a bug: https://jil.im/storkbug",
                error
            )
        };

        write!(f, "{}", desc)
    }
}
/**
 * Used to send metadata from WASM to JS. Derived from a `ParsedIndex` and
 * eventually serialized to JSON.
 */
#[derive(Serialize)]
pub struct IndexMetadata {
    #[serde(rename = "indexVersion")]
    index_version: String,
}

impl From<ParsedIndex> for IndexMetadata {
    fn from(index: ParsedIndex) -> Self {
        IndexMetadata {
            index_version: IndexVersion::from(index).to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum ParsedIndex {
    V2(v2::structs::Index),
    V3(v3::structs::Index),
}

impl std::convert::TryFrom<&IndexFromFile> for ParsedIndex {
    type Error = IndexParseError;

    fn try_from(index: &IndexFromFile) -> Result<ParsedIndex, IndexParseError> {
        fn parse_index_version(index: &IndexFromFile) -> Result<IndexVersion, IndexParseError> {
            if index.len() <= std::mem::size_of::<u64>() {
                return Err(IndexParseError::FileTooShort);
            }

            // This line can panic if `mid` > `index.len()`; hopefully the check above prevents a panic here.
            let (version_size_bytes, rest) = index.split_at(std::mem::size_of::<u64>());
            let version_size =
                u64::from_be_bytes(version_size_bytes.try_into().unwrap_or_default());

            let size_problem: Option<VersionSizeProblem> = match version_size {
                0..=1 => Some(VersionSizeProblem::Short),
                32..=u64::MAX => Some(VersionSizeProblem::Long),
                _ => None,
            };

            if let Some(size_problem) = size_problem {
                return Err(IndexParseError::BadVersionSize(version_size, size_problem));
            }

            if index.len()
                <= (std::mem::size_of::<u64>() as u64 + version_size)
                    .try_into()
                    .unwrap()
            {
                return Err(IndexParseError::FileTooShort);
            }

            #[allow(clippy::cast_possible_truncation)]
            let (version_bytes, _rest) = rest.split_at(version_size as usize);
            let version = String::from_utf8(version_bytes.to_vec());

            match version {
                Err(e) => Err(IndexParseError::VersionStringUtf8Error(e)),
                Ok(version) => match version.as_str() {
                    v2::VERSION_STRING => Ok(IndexVersion::V2),
                    v3::VERSION_STRING => Ok(IndexVersion::V3),
                    _ => Err(IndexParseError::UnknownVersionString(version)),
                },
            }
        }

        let parsed_index = match parse_index_version(index)? {
            IndexVersion::V2 => {
                let index = v2::structs::Index::from_file(index);
                ParsedIndex::V2(index)
            }

            IndexVersion::V3 => {
                let index = v3::structs::Index::try_from(index)?;
                ParsedIndex::V3(index)
            }
        };

        Ok(parsed_index)
    }
}

#[cfg(test)]
mod successfully_parse_historical_index_versions {
    use super::{IndexVersion, ParsedIndex};
    use std::convert::TryFrom;
    use std::fs;
    use std::io::{BufReader, Read};

    macro_rules! validate_version {
        ($file:expr, $version:expr) => {
            let file = fs::File::open($file).unwrap();
            let mut buf_reader = BufReader::new(file);
            let mut index_bytes: Vec<u8> = Vec::new();
            let _bytes_read = buf_reader.read_to_end(&mut index_bytes);
            let result = IndexVersion::from(ParsedIndex::try_from(index_bytes.as_slice()).unwrap());
            assert_eq!($version, result);
        };
    }

    #[test]
    fn can_get_version_of_0_5_3_index() {
        validate_version!(
            "./src/test-indexes/federalist-min-0.5.3.st",
            IndexVersion::V2
        );
    }

    #[test]
    fn can_get_version_of_0_6_0_index() {
        validate_version!(
            "./src/test-indexes/federalist-min-0.6.0.st",
            IndexVersion::V2
        );
    }

    #[test]
    fn can_get_version_of_1_0_0_index() {
        validate_version!(
            "./src/test-indexes/federalist-min-0.7.0.st",
            IndexVersion::V3
        );
    }
}

#[cfg(test)]
mod bad_blob_tests {
    use super::{IndexParseError, ParsedIndex, VersionSizeProblem};
    use std::convert::TryFrom;

    #[test]
    fn unknown_version_fails() {
        let badstring = "bad index".as_bytes();
        let err = ParsedIndex::try_from(badstring).unwrap_err();
        assert!(
            err == IndexParseError::BadVersionSize(
                7_089_057_378_828_444_773,
                VersionSizeProblem::Long
            ),
            "Bad error type, found {:?}",
            err
        );
    }

    #[test]
    fn short_blob_throws_short_error() {
        let bytes: &[u8] = &[0, 0, 0, 0, 255, 255, 255, 255];
        assert_eq!(
            ParsedIndex::try_from(bytes).unwrap_err(),
            IndexParseError::FileTooShort
        )
    }

    #[test]
    fn bad_index_version_throws_error() {
        let bytes: &[u8] = &[0, 0, 0, 0, 0, 0, 0, 0, 255];
        assert_eq!(
            ParsedIndex::try_from(bytes).unwrap_err(),
            IndexParseError::BadVersionSize(0, VersionSizeProblem::Short)
        )
    }
}
