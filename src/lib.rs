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

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    enum Cafebabe {
        Bar {
            byte_id: i8,
            position: (u64, u64),
        },
        Baz {
            id: u64,
            name: String,
        },
    }

    impl Identifiable for Cafebabe {
        fn get_id(&self) -> i32 {
            match *self {
                Cafebabe::Bar { .. } => 0x0badf00di32,
                Cafebabe::Baz { .. } => 0xbaaaaaadi32,
            }
        }

        fn get_enum_variant_id(&self) -> Option<u32> {
            let variant_id = match *self {
                Cafebabe::Bar { .. } => 0,
                Cafebabe::Baz { .. } => 1,
            };

            Some(variant_id)
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
            57, 0, 0, 0, 0, 0, 0, 0,    // 57 as little-endian 64-bit int
        ];

        static ref CAFEBABE_BAR: Cafebabe = Cafebabe::Bar {
            byte_id: -20,
            position: (350, 142857),
        };

        static ref CAFEBABE_BAR_SERIALIZED: Vec<u8> = vec![
            0x0d, 0xf0, 0xad, 0x0b,     // id of Cafebabe::Bar in little-endian
            236, 255, 255, 255,         // -20 as 32-bit int (MTProto doesn't support less than 32-bit)
            94, 1, 0, 0, 0, 0, 0, 0,    // 350 as little-endian 64-bit int
            9, 46, 2, 0, 0, 0, 0, 0,    // 142857 as little-endian 64-bit int
        ];

        static ref CAFEBABE_BAZ: Cafebabe = Cafebabe::Baz {
            id: u64::max_value(),
            name: "baz".to_owned(),
        };

        static ref CAFEBABE_BAZ_SERIALIZED: Vec<u8> = vec![
            0xad, 0xaa, 0xaa, 0xba,                    // id of Cafebabe::Baz in little-endian
            255, 255, 255, 255, 255, 255, 255, 255,    // u64::max_value() == 2 ** 64 - 1
            3, 98, 97, 122,                            // string "bar" of length 3
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


    #[test]
    fn test_enum_variant_to_vec() {
        let vec = to_vec(&*CAFEBABE_BAR).unwrap();

        assert_eq!(vec, *CAFEBABE_BAR_SERIALIZED);
    }

    #[test]
    fn test_enum_variant_to_writer() {
        let mut vec = Vec::new();
        to_writer(&mut vec, &*CAFEBABE_BAR).unwrap();

        assert_eq!(vec, *CAFEBABE_BAR_SERIALIZED);
    }

    #[test]
    fn test_enum_variant_from_slice() {
        let cafebabe_bar_deserialized: Cafebabe = from_slice(&*CAFEBABE_BAR_SERIALIZED, Some(0)).unwrap();

        assert_eq!(cafebabe_bar_deserialized, *CAFEBABE_BAR);
    }

    #[test]
    fn test_enum_variant_from_reader() {
        let cafebabe_bar_deserialized: Cafebabe = from_slice(CAFEBABE_BAR_SERIALIZED.as_slice(), Some(0)).unwrap();

        assert_eq!(cafebabe_bar_deserialized, *CAFEBABE_BAR);
    }


    #[test]
    fn test_enum_variant_to_vec2() {
        let vec = to_vec(&*CAFEBABE_BAZ).unwrap();

        assert_eq!(vec, *CAFEBABE_BAZ_SERIALIZED);
    }

    #[test]
    fn test_enum_variant_to_writer2() {
        let mut vec = Vec::new();
        to_writer(&mut vec, &*CAFEBABE_BAZ).unwrap();

        assert_eq!(vec, *CAFEBABE_BAZ_SERIALIZED);
    }

    #[test]
    fn test_enum_variant_from_slice2() {
        let cafebabe_baz_deserialized: Cafebabe = from_slice(&*CAFEBABE_BAZ_SERIALIZED, Some(1)).unwrap();

        assert_eq!(cafebabe_baz_deserialized, *CAFEBABE_BAZ);
    }

    #[test]
    fn test_enum_variant_from_reader2() {
        let cafebabe_baz_deserialized: Cafebabe = from_slice(CAFEBABE_BAZ_SERIALIZED.as_slice(), Some(1)).unwrap();

        assert_eq!(cafebabe_baz_deserialized, *CAFEBABE_BAZ);
    }


    #[test]
    fn test_serialization_alignment() {
        assert!(FOO_SERIALIZED.len() % 4 == 0);
        assert!(CAFEBABE_BAR_SERIALIZED.len() % 4 == 0);
        assert!(CAFEBABE_BAZ_SERIALIZED.len() % 4 == 0);
    }
}
