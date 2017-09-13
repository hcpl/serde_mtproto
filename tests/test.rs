//#[cfg(feature = "extprim")]
//extern crate extprim;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate pretty_assertions;
extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_derive;
extern crate serde_mtproto as serde_mtproto_other_name;    // Tests `serde_mtproto_derive`
#[macro_use]
extern crate serde_mtproto_derive;


use std::collections::BTreeMap;

//#[cfg(feature = "extprim")]
//use extprim::i128::i128;
use serde::de::{Deserializer, DeserializeSeed};
use serde_bytes::ByteBuf;
use serde_mtproto_other_name::{Boxed, MtProtoSized, UnsizedByteBuf, UnsizedByteBufSeed, to_bytes, to_writer, from_bytes, from_reader};


#[derive(Debug, PartialEq, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[id = "0xdeadbeef"]
struct Foo {
    has_receiver: bool,
    size: usize,
    raw_info: ByteBuf,
}

#[derive(Debug, PartialEq, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[id = "0x80808080"]
struct Message {
    auth_key_id: i64,
    msg_key: [u32; 4],
    #[serde(deserialize_with = "deserialize_message")]
    encrypted_data: UnsizedByteBuf,
}

fn deserialize_message<'de, D>(deserializer: D) -> Result<UnsizedByteBuf, D::Error>
    where D: Deserializer<'de>
{
    UnsizedByteBufSeed::new(19).deserialize(deserializer)
}

fn pad(bytes: &[u8]) -> Vec<u8> {
    let padding = (16 - bytes.len() % 16) % 16;
    let mut byte_buf = Vec::with_capacity(bytes.len() + padding);
    byte_buf.extend_from_slice(bytes);

    for _ in 0..padding {
        byte_buf.push(0);
    }

    byte_buf
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

    static ref MESSAGE: Message = Message {
        auth_key_id: -0x7edcba9876543210,
        msg_key: [3230999370, 1546177172, 3106848747, 2091612143],
        encrypted_data: UnsizedByteBuf::new(pad(&vec![0,1,2,3,4,5,6,7,8,9,10,11,12,13,14,15,16,17,18])),
    };

    static ref MESSAGE_SERIALIZED_BARE: Vec<u8> = vec![
        0xf0, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x81,
        0x4a, 0x23, 0x95, 0xc0, 0x94, 0xca, 0x28, 0x5c,
        0xeb, 0xbf, 0x2e, 0xb9, 0xef, 0x77, 0xab, 0x7c,
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        16, 17, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    ];

    static ref MESSAGE_SERIALIZED_BOXED: Vec<u8> = vec![
        0x80, 0x80, 0x80, 0x80,
        0xf0, 0xcd, 0xab, 0x89, 0x67, 0x45, 0x23, 0x81,
        0x4a, 0x23, 0x95, 0xc0, 0x94, 0xca, 0x28, 0x5c,
        0xeb, 0xbf, 0x2e, 0xb9, 0xef, 0x77, 0xab, 0x7c,
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15,
        16, 17, 18, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
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


macro_rules! test_suite {
    ($to_bytes:ident, $to_writer:ident, $from_bytes:ident, $from_reader:ident, $size_prediction:ident => 
     $Type:ty: ($SER_VAR:expr, $DE_VAR:expr, $VAR_SERIALIZED:expr,
                $var_deserialized:ident, $enum_variant_hint:expr, $var_deserialized_assert:expr)
    ) => {
        #[test]
        fn $to_bytes() {
            let vec = to_bytes(&$SER_VAR).unwrap();

            assert_eq!(vec, *$VAR_SERIALIZED);
        }

        #[test]
        fn $to_writer() {
            let mut vec = Vec::new();
            to_writer(&mut vec, &$SER_VAR).unwrap();

            assert_eq!(vec, *$VAR_SERIALIZED);
        }

        #[test]
        fn $from_bytes() {
            let $var_deserialized: $Type = from_bytes(&*$VAR_SERIALIZED, $enum_variant_hint).unwrap();

            assert_eq!($var_deserialized_assert, $DE_VAR);
        }

        #[test]
        fn $from_reader() {
            let $var_deserialized: $Type = from_reader($VAR_SERIALIZED.as_slice(), $enum_variant_hint).unwrap();

            assert_eq!($var_deserialized_assert, $DE_VAR);
        }

        #[test]
        fn $size_prediction() {
            let predicted_len = $SER_VAR.get_size_hint().unwrap();

            assert_eq!(predicted_len, $VAR_SERIALIZED.len());
        }
    };
}

macro_rules! test_suite_bare {
    ($to_bytes:ident, $to_writer:ident, $from_bytes:ident, $from_reader:ident, $size_prediction:ident => 
     $Type:ty: ($VAR:expr, $VAR_SERIALIZED_BARE:expr, $var_deserialized:ident, $enum_variant_hint:expr)
    ) => {
        test_suite!{
            $to_bytes, $to_writer, $from_bytes, $from_reader, $size_prediction =>
            $Type: (*$VAR, *$VAR, $VAR_SERIALIZED_BARE, $var_deserialized,
                    $enum_variant_hint, $var_deserialized)
        }
    };
}

macro_rules! test_suite_boxed {
    ($to_bytes:ident, $to_writer:ident, $from_bytes:ident, $from_reader:ident, $size_prediction:ident => 
     $Type:ty: ($VAR:expr, $VAR_SERIALIZED_BOXED:expr, $var_deserialized:ident, $enum_variant_hint:expr)
    ) => {
        test_suite!{
            $to_bytes, $to_writer, $from_bytes, $from_reader, $size_prediction =>
            Boxed<$Type>: (Boxed::new(&*$VAR), *$VAR, $VAR_SERIALIZED_BOXED,
                           $var_deserialized, $enum_variant_hint, $var_deserialized.into_inner())
        }
    };
}


test_suite_bare! {
    test_struct_to_bytes_bare,
    test_struct_to_writer_bare,
    test_struct_from_bytes_bare,
    test_struct_from_reader_bare,
    test_struct_size_prediction_bare =>
    Foo: (FOO, FOO_SERIALIZED_BARE, foo_deserialized_bare, None)
}

test_suite_boxed! {
    test_struct_to_bytes_boxed,
    test_struct_to_writer_boxed,
    test_struct_from_bytes_boxed,
    test_struct_from_reader_boxed,
    test_struct_size_prediction_boxed =>
    Foo: (FOO, FOO_SERIALIZED_BOXED, foo_deserialized_boxed, None)
}


test_suite_bare! {
    test_struct_to_bytes_bare2,
    test_struct_to_writer_bare2,
    test_struct_from_bytes_bare2,
    test_struct_from_reader_bare2,
    test_struct_size_prediction_bare2 =>
    Message: (MESSAGE, MESSAGE_SERIALIZED_BARE, message_deserialized_bare, None)
}

test_suite_boxed! {
    test_struct_to_bytes_boxed2,
    test_struct_to_writer_boxed2,
    test_struct_from_bytes_boxed2,
    test_struct_from_reader_boxed2,
    test_struct_size_prediction_boxed2 =>
    Message: (MESSAGE, MESSAGE_SERIALIZED_BOXED, message_deserialized_boxed, None)
}


test_suite_bare! {
    test_unit_struct_to_bytes_bare,
    test_unit_struct_to_writer_bare,
    test_unit_struct_from_bytes_bare,
    test_unit_struct_from_reader_bare,
    test_unit_struct_size_prediction_bare =>
    Nothing: (NOTHING, NOTHING_SERIALIZED_BARE, nothing_deserialized_bare, None)
}

test_suite_boxed! {
    test_unit_struct_to_bytes_boxed,
    test_unit_struct_to_writer_boxed,
    test_unit_struct_from_bytes_boxed,
    test_unit_struct_from_reader_boxed,
    test_unit_struct_size_prediction_boxed =>
    Nothing: (NOTHING, NOTHING_SERIALIZED_BOXED, nothing_deserialized_boxed, None)
}


test_suite_boxed! {
    test_enum_variant_to_bytes_boxed,
    test_enum_variant_to_writer_boxed,
    test_enum_variant_from_bytes_boxed,
    test_enum_variant_from_reader_boxed,
    test_enum_variant_size_prediction_boxed =>
    Cafebabe<u32>: (CAFEBABE_BAR, CAFEBABE_BAR_SERIALIZED_BOXED, cafebabe_bar_deserialized_boxed, Some("Bar"))
}


test_suite_boxed! {
    test_enum_variant_to_bytes_boxed2,
    test_enum_variant_to_writer_boxed2,
    test_enum_variant_from_bytes_boxed2,
    test_enum_variant_from_reader_boxed2,
    test_enum_variant_size_prediction_boxed2 =>
    Cafebabe<Vec<bool>>: (CAFEBABE_BAZ, CAFEBABE_BAZ_SERIALIZED_BOXED, cafebabe_baz_deserialized_boxed, Some("Baz"))
}


test_suite_boxed! {
    test_unit_enum_variant_to_bytes_boxed,
    test_unit_enum_variant_to_writer_boxed,
    test_unit_enum_variant_from_bytes_boxed,
    test_unit_enum_variant_from_reader_boxed,
    test_unit_enum_variant_size_prediction_boxed =>
    Cafebabe<()>: (CAFEBABE_BLOB, CAFEBABE_BLOB_SERIALIZED_BOXED, cafebabe_blob_deserialized_boxed, Some("Blob"))
}


/// MTProto-serialized data must be aligned by 4 bytes.
#[test]
fn test_serialization_alignment() {
    assert!(FOO_SERIALIZED_BARE.len() % 4 == 0);
    assert!(FOO_SERIALIZED_BOXED.len() % 4 == 0);
    assert!(NOTHING_SERIALIZED_BARE.len() % 4 == 0);
    assert!(NOTHING_SERIALIZED_BOXED.len() % 4 == 0);
    assert!(MESSAGE_SERIALIZED_BARE.len() % 4 == 0);
    assert!(MESSAGE_SERIALIZED_BOXED.len() % 4 == 0);
    assert!(CAFEBABE_BAR_SERIALIZED_BOXED.len() % 4 == 0);
    assert!(CAFEBABE_BAZ_SERIALIZED_BOXED.len() % 4 == 0);
    assert!(CAFEBABE_BLOB_SERIALIZED_BOXED.len() % 4 == 0);
}
