use std::fmt;

use serde::ser::{Serialize, Serializer};
use serde::de::{self, Deserialize, Deserializer, Visitor};


#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ByteBuf {
    byte_buf: Vec<u8>,
}

impl ByteBuf {
    pub fn new(byte_buf: Vec<u8>) -> ByteBuf {
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


#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Bytes<'a> {
    bytes: &'a [u8],
}

impl<'a> Bytes<'a> {
    pub fn new(bytes: &'a [u8]) -> Bytes<'a> {
        Bytes {
            bytes: bytes,
        }
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
