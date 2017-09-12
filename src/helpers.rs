//! Helper types for assisting in some [de]serialization scenarios.

use std::fmt;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::de::{self, Deserializer, DeserializeSeed, Visitor};
use serde::ser::{Serialize, Serializer, SerializeTuple};


/// A byte buffer which doesn'y write its length when serialized.
pub struct UnsizedByteBuf {
    inner: Vec<u8>,
}

impl UnsizedByteBuf {
    /// Construct a new unsized byte buffer seed with the length of the byte sequence to be
    /// deserialized.
    pub fn new(inner: Vec<u8>) -> UnsizedByteBuf {
        UnsizedByteBuf {
            inner: inner,
        }
    }

    /// Consumes the `UnsizedByteBuf` and returns the underlying byte buffer.
    pub fn into_inner(self) -> Vec<u8> {
        self.inner
    }
}

impl Serialize for UnsizedByteBuf {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where S: Serializer
    {
        let inner_len = self.inner.len();
        let padded_len = inner_len + (16 - inner_len % 16) % 16;
        let u64_padded_len = padded_len / 8;

        let mut inner_ref = self.inner.as_slice();
        let mut serialize_tuple = serializer.serialize_tuple(u64_padded_len)?;

        let mut serialized_len = 0;
        while serialized_len < padded_len {
            let u64_value = if inner_ref.len() > 0 {
                inner_ref.read_u64::<LittleEndian>().unwrap_or_else(|_| {
                    if let Ok(value) = inner_ref.read_uint::<LittleEndian>(inner_ref.len()) {
                        value
                    } else {
                        unreachable!("Should be able to read remaining bytes without errors from {:?}", inner_ref)
                    }
                })
            } else {
                0
            };

            serialize_tuple.serialize_element(&u64_value)?;
            serialized_len += 8;
        }

        serialize_tuple.end()
    }
}

/// An unsized byte buffer seed with the length of the byte sequence to be deserialized.
pub struct UnsizedByteBufSeed {
    inner_len: u32,
}

impl UnsizedByteBufSeed {
    /// Construct a new unsized byte buffer seed with the length of the byte sequence to be
    /// deserialized.
    pub fn new(inner_len: u32) -> UnsizedByteBufSeed {
        UnsizedByteBufSeed {
            inner_len: inner_len,
        }
    }
}

impl<'de> DeserializeSeed<'de> for UnsizedByteBufSeed {
    type Value = UnsizedByteBuf;

    fn deserialize<D>(self, deserializer: D) -> Result<UnsizedByteBuf, D::Error>
        where D: Deserializer<'de>
    {
        struct UnsizedByteBufVisitor;

        impl<'de> Visitor<'de> for UnsizedByteBufVisitor {
            type Value = UnsizedByteBuf;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a stream of bytes without prepended length and with a EOF")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<UnsizedByteBuf, A::Error>
                where A: de::SeqAccess<'de>
            {
                let mut inner = Vec::with_capacity(seq.size_hint().unwrap_or(0));

                while let Some(value) = seq.next_element()? {
                    if inner.write_u64::<LittleEndian>(value).is_err() {
                        unreachable!("Should be able to write into the in-memory buffer without errors to {:?}", inner)
                    }
                }

                Ok(UnsizedByteBuf {
                    inner: inner,
                })
            }
        }

        // FIXME: use safe cast here
        deserializer.deserialize_tuple(self.inner_len as usize / 8, UnsizedByteBufVisitor)
    }
}

impl MtProtoSized for UnsizedByteBuf {
    fn get_size_hint(&self) -> error:Result<usize> {
        Ok(self.len() + (16 - self.len() % 16) % 16)
    }
}
