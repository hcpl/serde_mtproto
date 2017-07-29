extern crate byteorder;
#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde;


mod common;

pub mod error;
pub mod ser;
pub mod de;

pub use error::{Error, Result};
pub use ser::{Serializer, to_vec, to_writer};
pub use de::{Deserializer, from_slice, from_reader};
