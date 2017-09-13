//! # Serde MTProto
//!
//! MTProto is a mobile-first protocol for access to a server API.
//! This crate provides means to serialize Rust types to its binary
//! representation and to deserialize from said representation.

#![deny(missing_docs)]


extern crate byteorder;
#[macro_use]
extern crate error_chain;
#[cfg(feature = "extprim")]
extern crate extprim;
#[macro_use]
extern crate log;
extern crate num_traits;
extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_derive;


mod utils;

pub mod boxed;
pub mod error;
pub mod helpers;
pub mod identifiable;
pub mod sized;
pub mod ser;
pub mod de;


// Reexport for convenience
pub use serde_bytes::{ByteBuf, Bytes};

pub use boxed::Boxed;
pub use error::{Error, ErrorKind, Result, ResultExt};
pub use helpers::{UnsizedByteBuf, UnsizedByteBufSeed};
pub use identifiable::Identifiable;
pub use sized::MtProtoSized;
pub use ser::{Serializer, to_bytes, to_writer, unsized_bytes_pad_to_bytes, unsized_bytes_pad_to_writer};
pub use de::{Deserializer, from_bytes, from_bytes_reuse, from_reader, from_reader_reuse};
