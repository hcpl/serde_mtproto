//! # Serde MTProto
//!
//! MTProto is a mobile-first protocol for access to a server API.
//! This crate provides means to serialize Rust types to its binary
//! representation and to deserialize from said representation.

extern crate byteorder;
#[macro_use]
extern crate error_chain;
extern crate extprim;
#[macro_use]
extern crate log;
extern crate num_traits;
extern crate serde;
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

pub use boxed::Boxed;
pub use error::{Error, ErrorKind, Result, ResultExt};
pub use helpers::{ByteBuf, Bytes};
pub use identifiable::Identifiable;
pub use sized::MtProtoSized;
pub use ser::{Serializer, to_bytes, to_writer};
pub use de::{Deserializer, from_bytes, from_reader};
