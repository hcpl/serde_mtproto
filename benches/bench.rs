#![feature(test)]


extern crate lipsum;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_mtproto;
#[macro_use]
extern crate serde_mtproto_derive;
extern crate test;


use serde_mtproto::{to_bytes, from_bytes};
use test::Bencher;


// PRIMITIVES

#[bench]
fn i64_serialize(b: &mut Bencher) {
    let random_i64: i64 = rand::random();

    b.iter(|| {
        to_bytes(&random_i64).unwrap();
    });
}

#[bench]
fn i64_deserialize(b: &mut Bencher) {
    let random_i64_serialized: [u8; 8] = rand::random();

    b.iter(|| {
        from_bytes::<i64>(&random_i64_serialized, None).unwrap();
    });
}

// STRINGS

#[bench]
fn string_empty_serialize(b: &mut Bencher) {
    b.iter(|| {
        to_bytes(&"").unwrap();
    });
}

#[bench]
fn string_empty_deserialize(b: &mut Bencher) {
    let string_serialized = to_bytes(&"").unwrap();

    b.iter(|| {
        from_bytes::<String>(&string_serialized, None).unwrap();
    });
}

#[bench]
fn string_short_serialize(b: &mut Bencher) {
    b.iter(|| {
        to_bytes(&"foobar").unwrap();
    });
}

#[bench]
fn string_short_deserialize(b: &mut Bencher) {
    let string_serialized = to_bytes(&"foobar").unwrap();

    b.iter(|| {
        from_bytes::<String>(&string_serialized, None).unwrap();
    });
}

#[bench]
fn string_medium_serialize(b: &mut Bencher) {
    b.iter(|| {
        to_bytes(&lipsum::LOREM_IPSUM).unwrap();
    });
}

#[bench]
fn string_medium_deserialize(b: &mut Bencher) {
    let string_serialized = to_bytes(&lipsum::LOREM_IPSUM).unwrap();

    b.iter(|| {
        from_bytes::<String>(&string_serialized, None).unwrap();
    });
}

#[bench]
fn string_long_serialize(b: &mut Bencher) {
    b.iter(|| {
        to_bytes(&lipsum::LIBER_PRIMUS).unwrap();
    });
}

#[bench]
fn string_long_deserialize(b: &mut Bencher) {
    let string_serialized = to_bytes(&lipsum::LIBER_PRIMUS).unwrap();

    b.iter(|| {
        from_bytes::<String>(&string_serialized, None).unwrap();
    });
}

fn random_string<R: rand::Rng>(rng: &mut R, max_words_count: usize) -> String {
    let lipsum_words_count: usize = rng.gen_range(0, max_words_count);

    lipsum::lipsum(lipsum_words_count)
}

#[bench]
fn random_string_serialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), 4096);

    b.iter(|| {
        to_bytes(&random_string).unwrap();
    });
}

#[bench]
fn random_string_deserialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), 4096);
    let random_string_serialized = to_bytes(&random_string).unwrap();

    b.iter(|| {
        from_bytes::<String>(&random_string_serialized, None).unwrap();
    });
}

// MISC

#[derive(Serialize, Deserialize, MtProtoIdentifiable)]
#[id = "0xd594ba98"]
struct Foo {
    bar: bool,
    s: String,
    group: (i16, u64, i8),
}

impl rand::Rand for Foo {
    fn rand<R: rand::Rng>(rng: &mut R) -> Foo {
        Foo {
            bar: rng.gen(),
            s: random_string(rng, 4096),
            group: rng.gen(),
        }
    }
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

#[bench]
fn random_foo_serialize(b: &mut Bencher) {
    let random_foo: Foo = rand::random();

    b.iter(|| {
        to_bytes(&random_foo).unwrap();
    });
}

#[bench]
fn random_foo_deserialize(b: &mut Bencher) {
    let random_foo: Foo = rand::random();
    let random_foo_serialized = to_bytes(&random_foo).unwrap();

    b.iter(|| {
        from_bytes::<Foo>(&random_foo_serialized, None).unwrap();
    });
}
