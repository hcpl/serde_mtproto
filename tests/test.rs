//#[cfg(feature = "extprim")]
//extern crate extprim;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate pretty_assertions;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_mtproto as serde_mtproto_other_name;    // Tests `serde_mtproto_derive`
#[macro_use]
extern crate serde_mtproto_derive;


use std::collections::BTreeMap;

//#[cfg(feature = "extprim")]
//use extprim::i128::i128;
use serde_mtproto_other_name::{Boxed, ByteBuf, MtProtoSized, to_bytes, to_writer, from_bytes, from_reader};


#[derive(Debug, PartialEq, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[id = "0xdeadbeef"]
struct Foo {
    has_receiver: bool,
    size: usize,
    raw_info: ByteBuf,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[id = "0xd15ea5e0"]
struct Nothing;

#[derive(Debug, PartialEq, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
enum Cafebabe<T: MtProtoSized> {
    #[id = "0x0badf00d"]
    Bar {
        byte_id: i8,
        position: (u64, u32),
        data: Boxed<T>,
        //#[cfg(feature = "extprim")]
        //bignum: i128,
        ratio: f32,
    },
    #[id = "0xbaaaaaad"]
    Baz {
        id: u64,
        name: String,
        payload: T,
        // not HashMap, because we need deterministic ordering for testing purposes
        mapping: BTreeMap<String, i64>,
    },
    #[id = "0x0d00d1e0"]
    Blob,
}


lazy_static! {
    static ref FOO: Foo = Foo {
        has_receiver: true,
        size: 57,
        raw_info: ByteBuf::from(vec![56, 114, 200, 1]),
    };

    static ref FOO_SERIALIZED_BARE: Vec<u8> = vec![
        181, 117, 114, 153,             // id of true in little-endian
        57, 0, 0, 0, 0, 0, 0, 0,        // 57 as little-endian 64-bit int
        4, 56, 114, 200, 1, 0, 0, 0,    // byte buffer containing 4 bytes
    ];

    static ref FOO_SERIALIZED_BOXED: Vec<u8> = vec![
        0xef, 0xbe, 0xad, 0xde,         // id of Foo in little-endian
        181, 117, 114, 153,             // id of true in little-endian
        57, 0, 0, 0, 0, 0, 0, 0,        // 57 as little-endian 64-bit int
        4, 56, 114, 200, 1, 0, 0, 0,    // byte buffer containing 4 bytes
    ];

    static ref NOTHING: Nothing = Nothing;

    static ref NOTHING_SERIALIZED_BARE: Vec<u8> = vec![];

    static ref NOTHING_SERIALIZED_BOXED: Vec<u8> = vec![
        0xe0, 0xa5, 0x5e, 0xd1,    // id of Nothing in little-endian
    ];

    static ref CAFEBABE_BAR: Cafebabe<u32> = Cafebabe::Bar {
        byte_id: -20,
        position: (350, 142857),
        data: Boxed::new(4096),
        // TODO: uncomment this after bumping minimal Rust version to 1.20 with the struct field
        // attributes feature.
        //
        //#[cfg(feature = "extprim")]
        //bignum: i128::from_str("100000000000000000000000000000000000000").unwrap(),
        ratio: 2.718281828,
    };

    static ref CAFEBABE_BAR_SERIALIZED_BOXED: Vec<u8> = vec![
        0x0d, 0xf0, 0xad, 0x0b,          // id of Cafebabe::Bar in little-endian
        236, 255, 255, 255,              // -20 as 32-bit int (MTProto doesn't support less than 32-bit)
        94, 1, 0, 0, 0, 0, 0, 0,         // 350 as little-endian 64-bit int
        9, 46, 2, 0,                     // 142857 as little-endian 32-bit int
        218, 155, 80, 168,               // id of int built-in MTProto type
        0, 16, 0, 0,                     // 4096 as little-endian 32-bit int
        0, 0, 0, 128, 10, 191, 5, 64,    // 2.718281828 as little-endian 32-bit floating point
    ];

    static ref CAFEBABE_BAZ: Cafebabe<Vec<bool>> = Cafebabe::Baz {
        id: u64::max_value(),
        name: "bee".to_owned(),
        payload: vec![false, true, false],
        mapping: btreemap!{
            "QWERTY".to_owned()     => -1048576,
            "something".to_owned()  => 0,
            "OtHeR".to_owned()      => 0x7fff_ffff_ffff_ffff,
            "こんにちは".to_owned() => 8024735636555,
            "".to_owned()           => -1,
        },
    };

    static ref CAFEBABE_BAZ_SERIALIZED_BOXED: Vec<u8> = vec![
        0xad, 0xaa, 0xaa, 0xba,                    // id of Cafebabe::Baz in little-endian
        255, 255, 255, 255, 255, 255, 255, 255,    // u64::max_value() == 2 ** 64 - 1
        3, 98, 101, 101,                           // string "bee" of length 4 and no padding

        3, 0, 0, 0,                                // vec has 3 elements, len as 32-bit int
        55, 151, 121, 188,                         // id of false in little-endian
        181, 117, 114, 153,                        // id of true in little-endian
        55, 151, 121, 188,                         // id of false in little-endian

        5, 0, 0, 0,                                // hashmap has 5 elements, len as 32-bit int
        0, 0, 0, 0,                                // ""
        255, 255, 255, 255, 255, 255, 255, 255,    // -1 as little-endian 64-bit int
        5, 79, 116, 72, 101, 82, 0, 0,             // "OtHeR"
        255, 255, 255, 255, 255, 255, 255, 127,    // 0x7fff_ffff_ffff_ffff as little-endian 64-bit int
        6, 81, 87, 69, 82, 84, 89, 0,              // "QWERTY"
        0, 0, 240, 255, 255, 255, 255, 255,        // -1048576 as little-endian 64-bit int
        9, 115, 111, 109, 101, 116, 104, 105, 110, 103, 0, 0,    // "something"
        0, 0, 0, 0, 0, 0, 0, 0,                    // 0 as little-endian 64-bit int
        15, 227, 129, 147, 227, 130, 147, 227, 129, 171, 227, 129, 161, 227, 129, 175,    // "こんにちは"
        75, 92, 132, 103, 76, 7, 0, 0,             // 8024735636555 as little-endian 64-bit int
    ];

    static ref CAFEBABE_BLOB: Cafebabe<()> = Cafebabe::Blob;

    static ref CAFEBABE_BLOB_SERIALIZED_BOXED: Vec<u8> = vec![
        0xe0, 0xd1, 0x00, 0x0d,    // id of Cafebabe::Blob in little-endian
    ];
}


#[test]
fn test_struct_to_bytes_bare() {
    let vec = to_bytes(&*FOO).unwrap();

    assert_eq!(vec, *FOO_SERIALIZED_BARE);
}

#[test]
fn test_struct_to_writer_bare() {
    let mut vec = Vec::new();
    to_writer(&mut vec, &*FOO).unwrap();

    assert_eq!(vec, *FOO_SERIALIZED_BARE);
}

#[test]
fn test_struct_from_bytes_bare() {
    let foo_deserialized: Foo = from_bytes(&*FOO_SERIALIZED_BARE, None).unwrap();

    assert_eq!(foo_deserialized, *FOO);
}

#[test]
fn test_struct_from_reader_bare() {
    let foo_deserialized: Foo = from_reader(FOO_SERIALIZED_BARE.as_slice(), None).unwrap();

    assert_eq!(foo_deserialized, *FOO);
}

#[test]
fn test_struct_size_prediction_bare() {
    let predicted_len = FOO.get_size_hint().unwrap();

    assert_eq!(predicted_len, FOO_SERIALIZED_BARE.len());
}


#[test]
fn test_struct_to_bytes_boxed() {
    let vec = to_bytes(&Boxed::new(&*FOO)).unwrap();

    assert_eq!(vec, *FOO_SERIALIZED_BOXED);
}

#[test]
fn test_struct_to_writer_boxed() {
    let mut vec = Vec::new();
    to_writer(&mut vec, &Boxed::new(&*FOO)).unwrap();

    assert_eq!(vec, *FOO_SERIALIZED_BOXED);
}

#[test]
fn test_struct_from_bytes_boxed() {
    let foo_deserialized_boxed: Boxed<Foo> = from_bytes(&*FOO_SERIALIZED_BOXED, None).unwrap();

    assert_eq!(foo_deserialized_boxed.into_inner(), *FOO);
}

#[test]
fn test_struct_from_reader_boxed() {
    let foo_deserialized_boxed: Boxed<Foo> = from_reader(FOO_SERIALIZED_BOXED.as_slice(), None).unwrap();

    assert_eq!(foo_deserialized_boxed.into_inner(), *FOO);
}

#[test]
fn test_struct_size_prediction_boxed() {
    let predicted_len = Boxed::new(&*FOO).get_size_hint().unwrap();

    assert_eq!(predicted_len, FOO_SERIALIZED_BOXED.len());
}


#[test]
fn test_unit_struct_to_bytes_bare() {
    let vec = to_bytes(&*NOTHING).unwrap();

    assert_eq!(vec, *NOTHING_SERIALIZED_BARE);
}

#[test]
fn test_unit_struct_to_writer_bare() {
    let mut vec = Vec::new();
    to_writer(&mut vec, &*NOTHING).unwrap();

    assert_eq!(vec, *NOTHING_SERIALIZED_BARE);
}

#[test]
fn test_unit_struct_from_bytes_bare() {
    let nothing_deserialized_bare: Nothing = from_bytes(&*NOTHING_SERIALIZED_BARE, None).unwrap();

    assert_eq!(nothing_deserialized_bare, *NOTHING);
}

#[test]
fn test_unit_struct_from_reader_bare() {
    let nothing_deserialized_bare: Nothing = from_reader(NOTHING_SERIALIZED_BARE.as_slice(), None).unwrap();

    assert_eq!(nothing_deserialized_bare, *NOTHING);
}

#[test]
fn test_unit_struct_size_prediction_bare() {
    let predicted_len = NOTHING.get_size_hint().unwrap();

    assert_eq!(predicted_len, NOTHING_SERIALIZED_BARE.len());
}


#[test]
fn test_unit_struct_to_bytes_boxed() {
    let vec = to_bytes(&Boxed::new(&*NOTHING)).unwrap();

    assert_eq!(vec, *NOTHING_SERIALIZED_BOXED);
}

#[test]
fn test_unit_struct_to_writer_boxed() {
    let mut vec = Vec::new();
    to_writer(&mut vec, &Boxed::new(&*NOTHING)).unwrap();

    assert_eq!(vec, *NOTHING_SERIALIZED_BOXED);
}

#[test]
fn test_unit_struct_from_bytes_boxed() {
    let nothing_deserialized_boxed: Boxed<Nothing> = from_bytes(&*NOTHING_SERIALIZED_BOXED, None).unwrap();

    assert_eq!(nothing_deserialized_boxed.into_inner(), *NOTHING);
}

#[test]
fn test_unit_struct_from_reader_boxed() {
    let nothing_deserialized_boxed: Boxed<Nothing> = from_reader(NOTHING_SERIALIZED_BOXED.as_slice(), None).unwrap();

    assert_eq!(nothing_deserialized_boxed.into_inner(), *NOTHING);
}

#[test]
fn test_unit_struct_size_prediction_boxed() {
    let predicted_len = Boxed::new(&*NOTHING).get_size_hint().unwrap();

    assert_eq!(predicted_len, NOTHING_SERIALIZED_BOXED.len());
}


#[test]
fn test_enum_variant_to_bytes_boxed() {
    let vec = to_bytes(&Boxed::new(&*CAFEBABE_BAR)).unwrap();

    assert_eq!(vec, *CAFEBABE_BAR_SERIALIZED_BOXED);
}

#[test]
fn test_enum_variant_to_writer_boxed() {
    let mut vec = Vec::new();
    to_writer(&mut vec, &Boxed::new(&*CAFEBABE_BAR)).unwrap();

    assert_eq!(vec, *CAFEBABE_BAR_SERIALIZED_BOXED);
}

#[test]
fn test_enum_variant_from_bytes_boxed() {
    let cafebabe_bar_deserialized_boxed: Boxed<Cafebabe<u32>> = from_bytes(&*CAFEBABE_BAR_SERIALIZED_BOXED, Some("Bar")).unwrap();

    assert_eq!(cafebabe_bar_deserialized_boxed.into_inner(), *CAFEBABE_BAR);
}

#[test]
fn test_enum_variant_from_reader_boxed() {
    let cafebabe_bar_deserialized_boxed: Boxed<Cafebabe<u32>> = from_reader(CAFEBABE_BAR_SERIALIZED_BOXED.as_slice(), Some("Bar")).unwrap();

    assert_eq!(cafebabe_bar_deserialized_boxed.into_inner(), *CAFEBABE_BAR);
}

#[test]
fn test_enum_variant_size_prediction_boxed() {
    let predicted_len = Boxed::new(&*CAFEBABE_BAR).get_size_hint().unwrap();

    assert_eq!(predicted_len, CAFEBABE_BAR_SERIALIZED_BOXED.len());
}


#[test]
fn test_enum_variant_to_bytes_boxed2() {
    let vec = to_bytes(&Boxed::new(&*CAFEBABE_BAZ)).unwrap();

    assert_eq!(vec, *CAFEBABE_BAZ_SERIALIZED_BOXED);
}

#[test]
fn test_enum_variant_to_writer_boxed2() {
    let mut vec = Vec::new();
    to_writer(&mut vec, &Boxed::new(&*CAFEBABE_BAZ)).unwrap();

    assert_eq!(vec, *CAFEBABE_BAZ_SERIALIZED_BOXED);
}

#[test]
fn test_enum_variant_from_bytes_boxed2() {
    let cafebabe_baz_deserialized_boxed: Boxed<Cafebabe<Vec<bool>>> = from_bytes(&*CAFEBABE_BAZ_SERIALIZED_BOXED, Some("Baz")).unwrap();

    assert_eq!(cafebabe_baz_deserialized_boxed.into_inner(), *CAFEBABE_BAZ);
}

#[test]
fn test_enum_variant_from_reader_boxed2() {
    let cafebabe_baz_deserialized_boxed: Boxed<Cafebabe<Vec<bool>>> = from_reader(CAFEBABE_BAZ_SERIALIZED_BOXED.as_slice(), Some("Baz")).unwrap();

    assert_eq!(cafebabe_baz_deserialized_boxed.into_inner(), *CAFEBABE_BAZ);
}

#[test]
fn test_enum_variant_size_prediction_boxed2() {
    let predicted_len = Boxed::new(&*CAFEBABE_BAZ).get_size_hint().unwrap();

    assert_eq!(predicted_len, CAFEBABE_BAZ_SERIALIZED_BOXED.len());
}


#[test]
fn test_unit_enum_variant_to_bytes_boxed() {
    let vec = to_bytes(&Boxed::new(&*CAFEBABE_BLOB)).unwrap();

    assert_eq!(vec, *CAFEBABE_BLOB_SERIALIZED_BOXED);
}

#[test]
fn test_unit_enum_variant_to_writer_boxed() {
    let mut vec = Vec::new();
    to_writer(&mut vec, &Boxed::new(&*CAFEBABE_BLOB)).unwrap();

    assert_eq!(vec, *CAFEBABE_BLOB_SERIALIZED_BOXED);
}

#[test]
fn test_unit_enum_variant_from_bytes_boxed() {
    let cafebabe_blob_deserialized_boxed: Boxed<Cafebabe<()>> = from_bytes(&*CAFEBABE_BLOB_SERIALIZED_BOXED, Some("Blob")).unwrap();

    assert_eq!(cafebabe_blob_deserialized_boxed.into_inner(), *CAFEBABE_BLOB);
}

#[test]
fn test_unit_enum_variant_from_reader_boxed() {
    let cafebabe_blob_deserialized_boxed: Boxed<Cafebabe<()>> = from_reader(CAFEBABE_BLOB_SERIALIZED_BOXED.as_slice(), Some("Blob")).unwrap();

    assert_eq!(cafebabe_blob_deserialized_boxed.into_inner(), *CAFEBABE_BLOB);
}

#[test]
fn test_unit_enum_variant_size_prediction_boxed() {
    let predicted_len = Boxed::new(&*CAFEBABE_BLOB).get_size_hint().unwrap();

    assert_eq!(predicted_len, CAFEBABE_BLOB_SERIALIZED_BOXED.len());
}


/// MTProto-serialized data must be aligned by 4 bytes.
#[test]
fn test_serialization_alignment() {
    assert!(FOO_SERIALIZED_BARE.len() % 4 == 0);
    assert!(FOO_SERIALIZED_BOXED.len() % 4 == 0);
    assert!(NOTHING_SERIALIZED_BARE.len() % 4 == 0);
    assert!(NOTHING_SERIALIZED_BOXED.len() % 4 == 0);
    assert!(CAFEBABE_BAR_SERIALIZED_BOXED.len() % 4 == 0);
    assert!(CAFEBABE_BAZ_SERIALIZED_BOXED.len() % 4 == 0);
    assert!(CAFEBABE_BLOB_SERIALIZED_BOXED.len() % 4 == 0);
}
