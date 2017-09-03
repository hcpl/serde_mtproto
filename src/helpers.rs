//! Various helper types to assist in expressing certain data layouts.

use std::fmt;

use serde::ser::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer, Visitor};

use error::{self, ErrorKind};
use sized::MtProtoSized;


/// A wrapper around `Vec<u8>` with `Serialize` and `Deserialize` implementations tailored
/// specifically for byte sequence [de]serialization.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ByteBuf {
    byte_buf: Vec<u8>,
}

impl ByteBuf {
    /// Wrap a byte buffer.
    pub fn new(byte_buf: Vec<u8>) -> ByteBuf {
        ByteBuf {
            byte_buf: byte_buf,
        }
    }

    /// Allocate a new `ByteBuf` from a `Bytes` instance.
    pub fn from_bytes<'a>(bytes: Bytes<'a>) -> ByteBuf {
        ByteBuf {
            byte_buf: bytes.bytes.to_vec(),
        }
    }

    /// Return an immutable reference to the underlying byte buffer.
    pub fn inner(&self) -> &Vec<u8> {
        &self.byte_buf
    }

    /// Return a mutable reference to the underlying byte buffer.
    pub fn inner_mut(&mut self) -> &mut Vec<u8> {
        &mut self.byte_buf
    }

    /// Unwrap the underlying byte buffer.
    pub fn into_inner(self) -> Vec<u8> {
        self.byte_buf
    }

    /// View a reference to `ByteBuf` as `Bytes`.
    pub fn as_bytes<'a>(&'a self) -> Bytes<'a> {
        Bytes {
            bytes: &self.byte_buf,
        }
    }
}

impl From<Vec<u8>> for ByteBuf {
    fn from(byte_buf: Vec<u8>) -> ByteBuf {
        ByteBuf {
            byte_buf: byte_buf,
        }
    }
}

impl Serialize for ByteBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_bytes(&self.byte_buf)
    }
}

impl<'de> Deserialize<'de> for ByteBuf {
    fn deserialize<D>(deserializer: D) -> Result<ByteBuf, D::Error>
        where D: Deserializer<'de>
    {
        struct ByteBufVisitor;

        impl<'de> Visitor<'de> for ByteBufVisitor {
            type Value = ByteBuf;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a Vec of u8")
            }

            fn visit_bytes<E>(self, v: &[u8]) -> Result<ByteBuf, E>
                where E: de::Error
            {
                Ok(ByteBuf::new(v.to_vec()))
            }

            fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<ByteBuf, E>
                where E: de::Error
            {
                Ok(ByteBuf::new(v))
            }
        }

        deserializer.deserialize_byte_buf(ByteBufVisitor)
    }
}

impl MtProtoSized for ByteBuf {
    fn get_size_hint(&self) -> error::Result<usize> {
        let len = self.byte_buf.len();

        let size = if len <= 253 {
            (len + 1) + (4 - (len + 1) % 4) % 4
        } else if len <= 0xff_ff_ff {
            len + (4 - len % 4) % 4
        } else {
            bail!(ErrorKind::ByteSeqTooLong(len));
        };

        Ok(size)
    }
}


/// A wrapper around `&[u8]` with `Serialize` and `Deserialize` implementations tailored
/// specifically for byte sequence [de]serialization.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Bytes<'a> {
    bytes: &'a [u8],
}

impl<'a> Bytes<'a> {
    /// Wrap a byte slice.
    pub fn new(bytes: &'a [u8]) -> Bytes<'a> {
        Bytes {
            bytes: bytes,
        }
    }

    /// View `Bytes` as a byte slice.
    pub fn as_inner(&self) -> &[u8] {
        self.bytes
    }
}

impl<'a> AsRef<[u8]> for Bytes<'a> {
    fn as_ref(&self) -> &[u8] {
        self.bytes
    }
}

impl<'a> Serialize for Bytes<'a> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        serializer.serialize_bytes(self.bytes)
    }
}

impl<'a, 'de: 'a> Deserialize<'de> for Bytes<'a> {
    fn deserialize<D>(deserializer: D) -> Result<Bytes<'a>, D::Error>
        where D: Deserializer<'de>
    {
        struct BytesVisitor;

        impl<'de> Visitor<'de> for BytesVisitor {
            type Value = Bytes<'de>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a slice of u8")
            }

            fn visit_borrowed_bytes<E>(self, v: &'de [u8]) -> Result<Bytes<'de>, E>
                where E: de::Error
            {
                Ok(Bytes::new(v))
            }
        }

        deserializer.deserialize_bytes(BytesVisitor)
    }
}

impl<'a> MtProtoSized for Bytes<'a> {
    fn get_size_hint(&self) -> error::Result<usize> {
        let len = self.bytes.len();

        let size = if len <= 253 {
            (len + 1) + (4 - (len + 1) % 4) % 4
        } else if len <= 0xff_ff_ff {
            len + (4 - len % 4) % 4
        } else {
            bail!(ErrorKind::ByteSeqTooLong(len));
        };

        Ok(size)
    }
}
