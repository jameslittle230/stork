use crate::index_versions::*;
use crate::IndexFromFile;
use std::convert::TryInto;
use std::error::Error;
use std::fmt;

pub fn parse_index_version(index: &IndexFromFile) -> Result<IndexVersion, IndexParseError> {
    let (version_size_bytes, rest) = index.split_at(std::mem::size_of::<u64>());
    let version_size = u64::from_be_bytes(version_size_bytes.try_into().unwrap_or_default());
    if version_size > 32 {
        return Err(IndexParseError::BadVersionSize(
            version_size,
            VersionSizeProblem::Long,
        ));
    }

    if version_size < 1 {
        return Err(IndexParseError::BadVersionSize(
            version_size,
            VersionSizeProblem::Short,
        ));
    }

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

#[derive(Debug, PartialEq)]
pub enum VersionSizeProblem {
    Short,
    Long,
}

#[derive(Debug, PartialEq)]
pub enum IndexParseError {
    FileTooShort,
    BadVersionSize(u64, VersionSizeProblem),
    VersionStringUtf8Error(std::string::FromUtf8Error),
    UnknownVersionString(String),
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

impl fmt::Display for IndexParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let desc: String = match self {
            IndexParseError::FileTooShort => {
                "Index file too short; could not find a version string.".to_string()
            }

            IndexParseError::BadVersionSize(size, problem) => format!(
                "Version size `{}` is too {}; this isn't a valid index file.",
                size,
                {
                    if problem == &VersionSizeProblem::Short {
                        "short"
                    } else {
                        "long"
                    }
                }
            ),
            IndexParseError::VersionStringUtf8Error(e) => format!(
                "Could not parse version string as valid UTF8, got:\n{:?}",
                e.as_bytes()
            ),
            IndexParseError::UnknownVersionString(string) => {
                format!("Unknown index version `{}` found", string)
            }
        };

        write!(f, "Could not parse index: {}", desc)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::{BufReader, Read};

    macro_rules! validate_version {
        ($file:expr, $version:expr) => {
            let file = fs::File::open($file).unwrap();
            let mut buf_reader = BufReader::new(file);
            let mut index_bytes: Vec<u8> = Vec::new();
            let _bytes_read = buf_reader.read_to_end(&mut index_bytes);
            let result = parse_index_version(index_bytes.as_slice()).unwrap();
            assert_eq!($version, result);
        };
    }

    #[test]
    fn unknown_version_fails() {
        let badstring = "bad index".as_bytes();
        let err = parse_index_version(badstring).unwrap_err();
        assert!(
            err == IndexParseError::BadVersionSize(7089057378828444773, VersionSizeProblem::Long),
            "Bad error type, found {:?}",
            err
        );
    }

    #[test]
    fn can_get_version_of_0_5_3_index() {
        validate_version!("./test-assets/federalist-min-0.5.3.st", IndexVersion::V2);
    }

    #[test]
    fn can_get_version_of_0_6_0_index() {
        validate_version!("./test-assets/federalist-min-0.6.0.st", IndexVersion::V2);
    }

    #[test]
    fn can_get_version_of_1_0_0_index() {
        validate_version!("./test-assets/federalist-min-1.0.0.st", IndexVersion::V3);
    }
}
