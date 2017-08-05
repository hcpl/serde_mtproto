extern crate byteorder;
#[macro_use]
extern crate error_chain;
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


#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod tests {
    use std::io;

    use ::{Identifiable, Serializer, to_vec, to_writer, from_slice, from_reader};


    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Foo {
        has_receiver: bool,
        size: usize,
    }

    impl Identifiable for Foo {
        fn get_id(&self) -> i32 {
            0xdeadbeefi32
        }

        fn get_enum_variant_id(&self) -> Option<u32> {
            None
        }
    }

    lazy_static! {
        static ref FOO: Foo = Foo {
            has_receiver: true,
            size: 57,
        };

        static ref FOO_SERIALIZED: Vec<u8> = vec![
            0xef, 0xbe, 0xad, 0xde,     // id of Foo in little-endian
            181, 117, 114, 153,         // id of true in little-endian
            57, 0, 0, 0, 0, 0, 0, 0,    // 57 as little-endian unsized 64-bit int
        ];
    }


    #[test]
    fn test_struct_to_vec() {
        let vec = to_vec(&*FOO).unwrap();

        assert_eq!(vec, *FOO_SERIALIZED);
    }

    #[test]
    fn test_struct_to_writer() {
        let mut vec = Vec::new();
        to_writer(&mut vec, &*FOO).unwrap();

        assert_eq!(vec, *FOO_SERIALIZED);
    }

    #[test]
    fn test_struct_from_slice() {
        let foo_deserialized: Foo = from_slice(&*FOO_SERIALIZED, None).unwrap();

        assert_eq!(foo_deserialized, *FOO);
    }

    #[test]
    fn test_struct_from_reader() {
        let foo_deserialized: Foo = from_reader(FOO_SERIALIZED.as_slice(), None).unwrap();

        assert_eq!(foo_deserialized, *FOO);
    }
}
