//! Helper types for assisting in some [de]serialization scenarios.

use std::fmt;
//use std::mem;

use byteorder::{ByteOrder, LittleEndian};
use serde::de::{self, Deserializer, DeserializeSeed, Error as DeError, Visitor};
use serde::ser::{Serialize, Serializer, SerializeTupleStruct};

use ::error::{self, DeErrorKind};
use ::sized::MtProtoSized;


// TODO: use `mem::size_of` const fn after bumping minimal Rust version to 1.22
//const CHUNK_SIZE: usize = mem::size_of::<u32>() / mem::size_of::<u8>();
const CHUNK_SIZE: usize = 4;


/// A byte buffer which doesn't write its length when serialized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsizedByteBuf {
    inner: Vec<u8>,
}

impl UnsizedByteBuf {
    /// Wrap a byte buffer.
    pub fn new(inner: Vec<u8>) -> error::Result<UnsizedByteBuf> {
        match inner.len() % 4 {
            0 => Ok(UnsizedByteBuf { inner }),
            _ => unimplemented!(),  // FIXME
        }
    }

    /// Create a new buffer and copy from `input` and pad so that the buffer
    /// length was divisible by 4.
    pub fn from_slice_pad(input: &[u8]) -> UnsizedByteBuf {
        let inner_len = input.len() + (4 - input.len() % 4) % 4;
        let mut inner = vec![0; inner_len];
        inner[0..input.len()].copy_from_slice(input);

        UnsizedByteBuf { inner }
    }

    /// Return an immutable reference to the underlying byte buffer.
    pub fn inner(&self) -> &Vec<u8> {
        &self.inner
    }

    /// Return a mutable reference to the underlying byte buffer.
    pub fn inner_mut(&mut self) -> &mut Vec<u8> {
        &mut self.inner
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
        let chunks_count = self.inner.len() / CHUNK_SIZE;

        assert!(self.inner.len() % CHUNK_SIZE == 0);

        let mut serialize_tuple = serializer.serialize_tuple_struct("UnsizedByteBuf", chunks_count)?;

        for chunk_u32 in self.inner.chunks(CHUNK_SIZE).map(LittleEndian::read_u32) {
            serialize_tuple.serialize_field(&chunk_u32)?;
        }

        serialize_tuple.end()
    }
}

impl MtProtoSized for UnsizedByteBuf {
    fn size_hint(&self) -> error::Result<usize> {
        Ok(self.inner.len())
    }
}

/// An unsized byte buffer seed with the length of the byte sequence to be deserialized.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnsizedByteBufSeed {
    inner_len: usize,
}

impl UnsizedByteBufSeed {
    /// Construct a new unsized byte buffer seed with the length of the byte sequence to be
    /// deserialized.
    pub fn new(inner_len: usize) -> error::Result<UnsizedByteBufSeed> {
        match inner_len % 4 {
            0 => Ok(UnsizedByteBufSeed { inner_len }),
            _ => unimplemented!(),  // FIXME
        }
    }
}

impl<'de> DeserializeSeed<'de> for UnsizedByteBufSeed {
    type Value = UnsizedByteBuf;

    fn deserialize<D>(self, deserializer: D) -> Result<UnsizedByteBuf, D::Error>
        where D: Deserializer<'de>
    {
        struct UnsizedByteBufVisitor {
            inner_len: usize,
        }

        impl<'de> Visitor<'de> for UnsizedByteBufVisitor {
            type Value = UnsizedByteBuf;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a stream of bytes without prepended length and with a EOF")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<UnsizedByteBuf, A::Error>
                where A: de::SeqAccess<'de>
            {
                let mut inner = vec![0; self.inner_len];

                assert!(self.inner_len % 4 == 0);
                let chunks_count = self.inner_len / CHUNK_SIZE;

                //TODO: add more info to error data
                let errconv = |kind: DeErrorKind| A::Error::custom(error::Error::from(kind));

                for (i, chunk_mut) in inner.chunks_mut(CHUNK_SIZE).enumerate() {
                    // FIXME: `usize` as `u32`
                    let chunk_u32 = seq.next_element()?
                        .ok_or_else(|| errconv(DeErrorKind::NotEnoughElements(i as u32, chunks_count as u32)))?;

                    LittleEndian::write_u32(chunk_mut, chunk_u32);
                }

                assert!(seq.next_element::<u32>()?.is_none());  // FIXME

                Ok(UnsizedByteBuf { inner })
            }
        }

        assert!(self.inner_len % 4 == 0);
        let chunks_count = self.inner_len / CHUNK_SIZE;

        deserializer.deserialize_tuple_struct(
            "UnsizedByteBuf",
            chunks_count,
            UnsizedByteBufVisitor { inner_len: self.inner_len },
        )
    }
}
