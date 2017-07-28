extern crate byteorder;
#[macro_use] extern crate error_chain;
extern crate serde;


pub mod error;
pub mod ser;
//pub mod de;

pub use error::{Error, Result};
pub use ser::{Serializer};
//pub use de::{Deserializer};
