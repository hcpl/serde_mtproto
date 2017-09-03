#![feature(test)]


extern crate lipsum;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_mtproto;
#[macro_use]
extern crate serde_mtproto_derive;
extern crate test;


use serde_mtproto::{to_bytes, from_bytes};
use test::Bencher;


#[derive(Serialize, Deserialize, MtProtoIdentifiable)]
#[id = "0xd594ba98"]
struct Foo {
    bar: bool,
    s: String,
    group: (i16, u64, i8),
}


#[bench]
fn i64_serialize(b: &mut Bencher) {
    b.iter(|| {
        to_bytes(&0x2a4ea54340204d12i64).unwrap();    // Truly random!
    });
}

#[bench]
fn i64_deserialize(b: &mut Bencher) {
    let i64_serialized = &[0xa3, 0xc6, 0x72, 0x47, 0x2f, 0xcd, 0xc5, 0xa4];    // Random too!

    b.iter(|| {
        from_bytes::<i64>(i64_serialized, None).unwrap();
    });
}

#[bench]
fn string_serialize(b: &mut Bencher) {
    b.iter(|| {
        to_bytes(&lipsum::LOREM_IPSUM).unwrap();
    });
}

#[bench]
fn string_deserialize(b: &mut Bencher) {
    let string_serialized = to_bytes(&lipsum::LOREM_IPSUM).unwrap();

    b.iter(|| {
        from_bytes::<String>(&string_serialized, None).unwrap();
    });
}

#[bench]
fn foo_serialize(b: &mut Bencher) {
    let foo = Foo {
        bar: false,
        s: "Hello, world!".to_owned(),
        group: (-500, 0xffff_ffff_ffff, -64),
    };

    b.iter(|| {
        to_bytes(&foo).unwrap();
    });
}

#[bench]
fn foo_deserialize(b: &mut Bencher) {
    let foo_serialized = [
        55, 151, 121, 188,
        13, 72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33, 0, 0,
        12, 254, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0,
        192, 255, 255, 255,
    ];

    b.iter(|| {
        from_bytes::<Foo>(&foo_serialized, None).unwrap();
    });
}
