use bstr::ByteSlice;
use bytes::{Buf, BufMut, Bytes, BytesMut};

/// Prefix: one of a few known values.
/// Bytes: decoded Length-Value blobs.
#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) struct Envelope {
    pub(crate) prefix: Prefix,
    pub(crate) bytes: Bytes,
}

#[cfg_attr(test, derive(Debug, PartialEq))]
pub(crate) enum Prefix {
    StorkV4Root,
    StorkV4Part,
}

#[cfg_attr(test, derive(PartialEq, Eq))]
#[cfg_attr(feature = "build", derive(Debug))]
pub enum EnvelopeDecodeError {
    TooShort,
    BadSegmentSize(u64),
    BadPrefixSize(u64),
    UnknownPrefix(),
}

impl Prefix {
    fn to_string(&self) -> &'static str {
        match self {
            Prefix::StorkV4Root => "stork-4",
            Prefix::StorkV4Part => "stork-4-part", // TODO: Figure out the different V4 parts that might exist and come up with different prefixes for them
        }
    }

    fn from_str(to_string: &str) -> Result<Self, EnvelopeDecodeError> {
        match to_string {
            "stork-4" => Ok(Self::StorkV4Root),
            "stork-4-part" => Ok(Self::StorkV4Part),
            _ => Err(EnvelopeDecodeError::UnknownPrefix()),
        }
    }
}

impl Envelope {
    pub(crate) fn wrap(prefix: Prefix, bytes: Bytes) -> Self {
        Self { prefix, bytes }
    }
}

impl Envelope {
    pub(crate) fn to_bytes(&self) -> Bytes {
        let version_bytes = Bytes::from(self.prefix.to_string());
        let bytes_capacity = self.bytes.len();

        let mut buf = BytesMut::with_capacity(
            bytes_capacity
                + version_bytes.len()
                + ((self.bytes.len() + 3) * std::mem::size_of::<u64>()),
        );

        buf.put_u8(version_bytes.len() as u8);
        buf.put(version_bytes);

        buf.put_u64(self.bytes.len() as u64);
        buf.put(self.bytes.clone());

        buf.freeze()
    }
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

        let length = buffer.get_u64().try_into().unwrap();
        let bytes = buffer.split_to(length);

        Ok(Self { prefix, bytes })
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
    fn happy_path_v4_parse() {
        let bytes = hexbytes(&hex!("07 73746F726B2D34 0000000000000001 00"));
        let versioned_index = Envelope::try_from(bytes);
        assert_eq!(
            versioned_index,
            Ok(Envelope {
                prefix: Prefix::StorkV4Root,
                bytes: hexbytes(&hex!("00"))
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
                prefix: Prefix::StorkV4Root,
                bytes: hexbytes(&hex!("00"))
            })
        );
    }

    #[test]
    fn happy_path_v4_write() {
        let bytes = hexbytes(&hex!("07 73746F726B2D34 0000000000000001 00"));
        let written_bytes = Envelope {
            prefix: Prefix::StorkV4Root,
            bytes: hexbytes(&hex!("00")),
        }
        .to_bytes();

        assert_eq!(bytes, written_bytes);
    }

    #[test]
    fn ascii_string_does_not_parse() {
        let bytes = Bytes::try_from("this is not an index".as_bytes()).unwrap();
        assert_eq!(
            Envelope::try_from(bytes).unwrap_err(),
            EnvelopeDecodeError::BadPrefixSize(116) // 't' -> 0x74 -> 0d116
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
            EnvelopeDecodeError::UnknownPrefix()
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
            EnvelopeDecodeError::UnknownPrefix()
        );
    }
}
