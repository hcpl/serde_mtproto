//! Helper types for assisting in some [de]serialization scenarios.

use std::fmt;

use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use serde::de::{self, Deserializer, DeserializeSeed, Error as DeError, Visitor};
use serde::ser::{Serialize, Serializer, SerializeTuple};

use error;
use sized::MtProtoSized;
use utils::safe_uint_cast;


/// A byte buffer which doesn't write its length when serialized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsizedByteBuf {
    inner: Vec<u8>,
}

impl UnsizedByteBuf {
    /// Wrap a byte buffer.
    pub fn new(inner: Vec<u8>) -> UnsizedByteBuf {
        UnsizedByteBuf { inner }
    }

    /// Consume the `UnsizedByteBuf` and return the underlying byte buffer.
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
            let u64_value = if !inner_ref.is_empty() {
                // Prefer using `WriteBytesExt::write_u64` over `ByteOrder::write_u64` in case
                // if something goes really wrong
                inner_ref.read_u64::<LittleEndian>().unwrap_or_else(|_| {
                    // TODO: Revise this part for guarantees of len % 4 == 0 or the like
                    // Must be the last element out there, let's read the truncated value and use
                    // it as any u64 element
                    if let Ok(value) = inner_ref.read_uint::<LittleEndian>(inner_ref.len()) {
                        value
                    } else {
                        // Prefer using a detailed error message in case if something goes really wrong
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

impl MtProtoSized for UnsizedByteBuf {
    fn size_hint(&self) -> error::Result<usize> {
        size_hint_from_unsized_byte_seq_len(self.inner.len())
    }
}

/// An unsized byte buffer seed with the length of the byte sequence to be deserialized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsizedByteBufSeed {
    inner_len: u32,
}

impl UnsizedByteBufSeed {
    /// Construct a new unsized byte buffer seed with the length of the byte sequence to be
    /// deserialized.
    pub fn new(inner_len: u32) -> UnsizedByteBufSeed {
        UnsizedByteBufSeed { inner_len }
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
                    // Prefer using `WriteBytesExt::write_u64` over `ByteOrder::write_u64` in case
                    // if something goes really wrong
                    if inner.write_u64::<LittleEndian>(value).is_err() {
                        unreachable!("Should be able to write without errors into the in-memory buffer {:?}", inner)
                    }
                }

                Ok(UnsizedByteBuf { inner })
            }
        }

        let padded_bytes_len = self.inner_len + (16 - self.inner_len % 16) % 16;
        let padded_len = safe_uint_cast::<u32, usize>(padded_bytes_len / 8).map_err(D::Error::custom)?;

        deserializer.deserialize_tuple(padded_len, UnsizedByteBufVisitor)
    }
}


/// A bytes slice which doesn't write its length when serialized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsizedBytes<'a> {
    inner: &'a [u8],
}

impl<'a> UnsizedBytes<'a> {
    /// Wrap a bytes slice.
    pub fn new(inner: &'a [u8]) -> UnsizedBytes<'a> {
        UnsizedBytes { inner }
    }

    /// View the `UnsizedBytes` as the underlying bytes slice.
    pub fn as_inner(&'a self) -> &'a [u8] {
        self.inner
    }
}

impl<'a> MtProtoSized for UnsizedBytes<'a> {
    fn size_hint(&self) -> error::Result<usize> {
        size_hint_from_unsized_byte_seq_len(self.inner.len())
    }
}


/// Helper function for everything naturally representable as a byte sequence.
///
/// This version **doesn't take** into account the byte sequence length since it is not contained
/// in the serialized representation of the byte sequence.
pub fn size_hint_from_unsized_byte_seq_len(len: usize) -> error::Result<usize> {
    let size = len + (16 - len % 16) % 16;
    assert!(size % 16 == 0);

    Ok(size)
}
