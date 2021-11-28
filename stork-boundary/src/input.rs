use bytes::{Buf, Bytes};
use thiserror::Error;

#[derive(Debug, PartialEq)]
pub enum VersionedIndex {
    V2(Bytes),
    V3(Bytes),
}

impl TryFrom<Bytes> for VersionedIndex {
    type Error = IndexVersioningError;

    fn try_from(value: Bytes) -> Result<Self, Self::Error> {
        let mut buffer = value;
        let u64_size = std::mem::size_of::<u64>();

        if buffer.len() <= u64_size {
            return Err(IndexVersioningError::FileTooShort);
        }

        let version_size = {
            let version_size = buffer.get_u64();
            let version_size: usize = version_size
                .try_into()
                .map_err(|_| IndexVersioningError::BadSegmentSize(version_size))?;

            if !(1..=32).contains(&version_size) {
                return Err(IndexVersioningError::BadVersionSize(
                    version_size.try_into().unwrap(),
                ));
            }

            Ok::<usize, IndexVersioningError>(version_size)
        }?;

        if buffer.len() < version_size {
            return Err(IndexVersioningError::FileTooShort);
        }

        let version_string = {
            let split = buffer.split_to(version_size);
            String::from_utf8(Vec::from(split.as_ref()))
        }?;

        match version_string.as_str() {
            "stork-2" => Ok(VersionedIndex::V2(buffer)),
            "stork-3" => {
                let index_size = {
                    let index_size = buffer.get_u64();
                    let index_size: usize = index_size
                        .try_into()
                        .map_err(|_| IndexVersioningError::BadSegmentSize(index_size))?;
                    Ok::<usize, IndexVersioningError>(index_size)
                }?;

                let index_bytes = buffer.split_to(index_size);

                Ok(VersionedIndex::V3(index_bytes))
            }
            _ => Err(IndexVersioningError::UnknownVersionString(version_string)),
        }
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum IndexVersioningError {
    #[error("Invalid index: index is too short and its version could not be determined.")]
    FileTooShort,

    #[error("Invalid index: found segment size `{0}`")]
    BadSegmentSize(u64),

    #[error("Invalid index: found version string that is `{0}` bytes long. The version string must be between 1 and 32 bytes long.")]
    BadVersionSize(u64),

    #[error(
        "Invalid index: could not parse version string as valid UTF8. Stork recieved error `{0}`"
    )]
    VersionStringUtf8Error(#[from] std::string::FromUtf8Error),

    #[error("Invalid index: unknown index version. Got `{0}`")]
    UnknownVersionString(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use pretty_assertions::assert_eq;

    #[test]
    fn happy_path_v2_parse() {
        let bytes = Bytes::try_from(hex!("0000000000000007 73746F726B2D32 00").as_ref()).unwrap();
        let versioned_index = VersionedIndex::try_from(bytes).unwrap();
        assert_eq!(
            versioned_index,
            VersionedIndex::V2(Bytes::try_from(hex!("00").as_ref()).unwrap())
        )
    }

    #[test]
    fn happy_path_v3_parse() {
        let bytes =
            Bytes::try_from(hex!("0000000000000007 73746F726B2D33 0000000000000001 00").as_ref())
                .unwrap();
        let versioned_index = VersionedIndex::try_from(bytes).unwrap();
        assert_eq!(
            versioned_index,
            VersionedIndex::V3(Bytes::try_from(hex!("00").as_ref()).unwrap())
        )
    }

    #[test]
    fn ascii_string_does_not_parse() {
        let bytes = Bytes::try_from("this is not an index".as_bytes()).unwrap();
        assert_eq!(
            VersionedIndex::try_from(bytes).unwrap_err(),
            IndexVersioningError::BadVersionSize(8_388_070_249_163_485_984)
        )
    }

    #[test]
    fn stated_33_byte_version_does_not_parse() {
        let bytes = Bytes::try_from(hex!("00000000 00000021 00").as_ref()).unwrap();
        assert_eq!(
            VersionedIndex::try_from(bytes).unwrap_err(),
            IndexVersioningError::BadVersionSize(33)
        )
    }

    #[test]
    fn stated_32_byte_version_parses() {
        let bytes = Bytes::try_from(hex!("00000000 00000020 00").as_ref()).unwrap();
        assert_eq!(
            VersionedIndex::try_from(bytes).unwrap_err(),
            IndexVersioningError::FileTooShort // Because the version string isn't 32 bytes long
        )
    }

    #[test]
    fn stated_1_byte_version_parses() {
        let bytes = Bytes::try_from(hex!("00000000 00000001 00").as_ref()).unwrap();
        assert_eq!(
            VersionedIndex::try_from(bytes).unwrap_err(),
            IndexVersioningError::UnknownVersionString("\x00".into())
        )
    }

    #[test]
    fn stated_0_byte_version_does_not_parse() {
        let bytes = Bytes::try_from(hex!("00000000 00000000 00").as_ref()).unwrap();
        assert_eq!(
            VersionedIndex::try_from(bytes).unwrap_err(),
            IndexVersioningError::BadVersionSize(0)
        )
    }

    #[test]
    fn short_index_does_not_parse() {
        let bytes = Bytes::try_from(hex!("000000000000FF").as_ref()).unwrap();
        assert_eq!(
            VersionedIndex::try_from(bytes).unwrap_err(),
            IndexVersioningError::FileTooShort
        )
    }

    #[test]
    fn invalid_utf8_version_does_not_parse() {
        let bytes = Bytes::try_from(hex!("0000000000000004 F0288CBC").as_ref()).unwrap();

        // This is an invalid 4-octet sequence where the second octet is invalid,
        // according to https://www.php.net/manual/en/reference.pcre.pattern.modifiers.php#54805
        let utf8_error = String::from_utf8(hex!("f0 28 8c bc").as_ref().to_vec()).unwrap_err();
        assert_eq!(
            VersionedIndex::try_from(bytes.clone()).unwrap_err(),
            IndexVersioningError::VersionStringUtf8Error(utf8_error)
        );

        assert_eq!(
            VersionedIndex::try_from(bytes).unwrap_err().to_string(),
            "Invalid index: could not parse version string as valid UTF8. Stork recieved error `invalid utf-8 sequence of 1 bytes from index 0`"
        );
    }
}
