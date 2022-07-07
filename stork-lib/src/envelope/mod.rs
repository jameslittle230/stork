use std::str::FromStr;

use bstr::ByteSlice;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use strum::ParseError;
use strum_macros::{Display, EnumString};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Envelope {
    pub(super) prefix: Prefix,
    pub(super) bytes: Vec<Bytes>,
}

#[derive(Debug, Clone, Display, PartialEq, Eq, EnumString)]
pub(crate) enum Prefix {
    #[strum(serialize = "stork-2")]
    StorkV2,

    #[strum(serialize = "stork-3")]
    StorkV3,

    #[strum(serialize = "stork-4")]
    StorkV4,
}

impl Envelope {
    pub(crate) fn wrap(prefix: Prefix, bytes: Vec<Bytes>) -> Self {
        Self { prefix, bytes }
    }
}

impl Envelope {
    pub(crate) fn to_bytes(&self) -> Bytes {
        let version_bytes = Bytes::from(self.prefix.to_string());
        let bytes_capacity = self.bytes.iter().fold(0, |acc, elem| acc + elem.len());

        let mut buf = BytesMut::with_capacity(
            bytes_capacity
                + version_bytes.len()
                + ((self.bytes.len() + 3) * std::mem::size_of::<u64>()),
        );

        if self.prefix == Prefix::StorkV2 || self.prefix == Prefix::StorkV3 {
            buf.put_u64(version_bytes.len() as u64);
        } else {
            buf.put_u8(version_bytes.len() as u8);
        }
        buf.put(version_bytes);

        for blob in &self.bytes {
            buf.put_u64(blob.len() as u64);
            buf.put(blob.clone());
        }

        buf.freeze()
    }
}

#[derive(Error, Debug, PartialEq)]
pub enum EnvelopeDecodeError {
    #[error("Invalid index: index is too short and its version could not be determined.")]
    TooShort,

    #[error("Invalid index: found segment size `{0}`")]
    BadSegmentSize(u64),

    #[error("Invalid index: found version string that is `{0}` bytes long. The version string must be between 1 and 32 bytes long.")]
    BadPrefixSize(u64),

    #[error("Invalid index: unknown index version. {0}")]
    UnknownPrefix(#[from] ParseError),
}

impl TryFrom<Bytes> for Envelope {
    type Error = EnvelopeDecodeError;

    fn try_from(mut buffer: Bytes) -> Result<Self, Self::Error> {
        let u64_size = std::mem::size_of::<u64>();

        if buffer.len() <= u64_size {
            return Err(EnvelopeDecodeError::TooShort);
        }

        let prefix_size = {
            let prefix_size: usize = if buffer.slice(0..=1).get_u8() == 0 {
                let prefix_size_u64 = buffer.get_u64();
                prefix_size_u64
                    .try_into()
                    .map_err(|_| EnvelopeDecodeError::BadSegmentSize(prefix_size_u64))?
            } else {
                buffer.get_u8().into()
            };

            if !(1..=32).contains(&prefix_size) {
                return Err(EnvelopeDecodeError::BadPrefixSize(
                    prefix_size.try_into().unwrap(),
                ));
            }

            Ok::<usize, EnvelopeDecodeError>(prefix_size)
        }?;

        if buffer.len() < prefix_size {
            return Err(EnvelopeDecodeError::TooShort);
        }

        let version_string = buffer.split_to(prefix_size);

        let prefix = Prefix::from_str(&version_string.as_bstr().to_string())?;

        // Special case - this was a mistake.
        if prefix == Prefix::StorkV2 {
            return Ok(Self {
                prefix,
                bytes: vec![buffer],
            });
        }

        let mut blobs: Vec<Bytes> = Vec::new();
        while !buffer.is_empty() {
            let length = buffer.get_u64().try_into().unwrap();
            blobs.push(buffer.split_to(length));
        }

        Ok(Self {
            prefix,
            bytes: blobs,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hex_literal::hex;
    use pretty_assertions::assert_eq;

    fn hexbytes(slice: &'static [u8]) -> Bytes {
        Bytes::try_from(slice).unwrap()
    }

    #[test]
    fn happy_path_v2_parse() {
        let bytes = hexbytes(&hex!("0000000000000007 73746F726B2D32 00"));
        let versioned_index = Envelope::try_from(bytes).unwrap();
        assert_eq!(
            versioned_index,
            Envelope {
                prefix: Prefix::StorkV2,
                bytes: vec![hexbytes(&hex!("00"))]
            }
        );
    }

    #[test]
    fn happy_path_v3_parse() {
        let bytes = hexbytes(&hex!("0000000000000007 73746F726B2D33 0000000000000001 00"));
        let versioned_index = Envelope::try_from(bytes).unwrap();
        assert_eq!(
            versioned_index,
            Envelope {
                prefix: Prefix::StorkV3,
                bytes: vec![hexbytes(&hex!("00"))]
            }
        );
    }

    #[test]
    fn happy_path_v4_parse() {
        let bytes = hexbytes(&hex!("07 73746F726B2D34 0000000000000001 00"));
        let versioned_index = Envelope::try_from(bytes);
        assert_eq!(
            versioned_index,
            Ok(Envelope {
                prefix: Prefix::StorkV4,
                bytes: vec![hexbytes(&hex!("00"))]
            })
        );
    }

    #[test]
    fn happy_path_v4_parse_long_prefix_size() {
        let bytes = hexbytes(&hex!("0000000000000007 73746F726B2D34 0000000000000001 00"));
        let versioned_index = Envelope::try_from(bytes);
        assert_eq!(
            versioned_index,
            Ok(Envelope {
                prefix: Prefix::StorkV4,
                bytes: vec![hexbytes(&hex!("00"))]
            })
        );
    }

    #[test]
    fn happy_path_v4_write() {
        let bytes = hexbytes(&hex!("07 73746F726B2D34 0000000000000001 00"));
        let written_bytes = Envelope {
            prefix: Prefix::StorkV4,
            bytes: vec![hexbytes(&hex!("00"))],
        }
        .to_bytes();

        assert_eq!(bytes, written_bytes);
    }

    #[test]
    fn ascii_string_does_not_parse() {
        let bytes = Bytes::try_from("this is not an index".as_bytes()).unwrap();
        assert_eq!(
            Envelope::try_from(bytes).unwrap_err(),
            EnvelopeDecodeError::BadPrefixSize(8_388_070_249_163_485_984)
        );
    }

    #[test]
    fn stated_33_byte_version_does_not_parse() {
        let bytes = Bytes::try_from(hex!("00000000 00000021 00").as_ref()).unwrap();
        assert_eq!(
            Envelope::try_from(bytes).unwrap_err(),
            EnvelopeDecodeError::BadPrefixSize(33)
        );
    }

    #[test]
    fn stated_32_byte_version_parses() {
        let bytes = Bytes::try_from(hex!("00000000 00000020 00").as_ref()).unwrap();
        assert_eq!(
            Envelope::try_from(bytes).unwrap_err(),
            EnvelopeDecodeError::TooShort // Because the version string isn't 32 bytes long
        );
    }

    #[test]
    fn stated_1_byte_version_parses() {
        let bytes = Bytes::try_from(hex!("00000000 00000001 00").as_ref()).unwrap();
        assert_eq!(
            Envelope::try_from(bytes).unwrap_err(),
            EnvelopeDecodeError::UnknownPrefix(ParseError::VariantNotFound)
        );
    }

    #[test]
    fn stated_0_byte_version_does_not_parse() {
        let bytes = Bytes::try_from(hex!("00000000 00000000 00").as_ref()).unwrap();
        assert_eq!(
            Envelope::try_from(bytes).unwrap_err(),
            EnvelopeDecodeError::BadPrefixSize(0)
        );
    }

    #[test]
    fn short_index_does_not_parse() {
        let bytes = Bytes::try_from(hex!("000000000000FF").as_ref()).unwrap();
        assert_eq!(
            Envelope::try_from(bytes).unwrap_err(),
            EnvelopeDecodeError::TooShort
        );
    }

    #[test]
    fn invalid_utf8_version_does_not_crash() {
        let bytes = Bytes::try_from(hex!("0000000000000004 F0288CBC").as_ref()).unwrap();

        assert_eq!(
            Envelope::try_from(bytes).unwrap_err(),
            EnvelopeDecodeError::UnknownPrefix(ParseError::VariantNotFound)
        );
    }
}
