extern crate byteorder;
#[macro_use] extern crate error_chain;
extern crate serde;


mod error;
mod ser;
//mod de;

pub use error::{Error, Result};
pub use ser::{Serializer};
//pub use de::{Deserializer};
