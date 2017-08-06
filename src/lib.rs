extern crate byteorder;
#[macro_use]
extern crate error_chain;
extern crate num_traits;
extern crate serde;
#[macro_use]
extern crate serde_derive;


mod common;

pub mod error;
pub mod identifiable;
pub mod ser;
pub mod de;

pub use error::{Error, Result};
pub use identifiable::Identifiable;
pub use ser::{Serializer, to_vec, to_writer};
pub use de::{Deserializer, from_slice, from_reader};
