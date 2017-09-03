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


use rand::{Rand, Rng};
use serde_mtproto::{to_bytes, from_bytes};
use test::Bencher;


// PRIMITIVES

#[bench]
fn bool_serialize(b: &mut Bencher) {
    let random_bool: bool = rand::random();

    b.iter(|| {
        to_bytes(&random_bool).unwrap();
    });
}

#[bench]
fn bool_deserialize(b: &mut Bencher) {
    let random_bool: bool = rand::random();
    let random_bool_serialized = to_bytes(&random_bool).unwrap();

    b.iter(|| {
        from_bytes::<bool>(&random_bool_serialized, None).unwrap();
    });
}

#[bench]
fn u8_serialize(b: &mut Bencher) {
    let random_u8: u8 = rand::random();

    b.iter(|| {
        to_bytes(&random_u8).unwrap();
    });
}

#[bench]
fn u8_deserialize(b: &mut Bencher) {
    let random_u8_serialized: [u8; 4] = [rand::random(), 0, 0, 0];

    b.iter(|| {
        from_bytes::<u8>(&random_u8_serialized, None).unwrap();
    });
}

#[bench]
fn u32_serialize(b: &mut Bencher) {
    let random_u32: u32 = rand::random();

    b.iter(|| {
        to_bytes(&random_u32).unwrap();
    });
}

#[bench]
fn u32_deserialize(b: &mut Bencher) {
    let random_u32_serialized: [u8; 4] = rand::random();

    b.iter(|| {
        from_bytes::<u32>(&random_u32_serialized, None).unwrap();
    });
}

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

#[bench]
fn u64_serialize(b: &mut Bencher) {
    let random_u64: u64 = rand::random();

    b.iter(|| {
        to_bytes(&random_u64).unwrap();
    });
}

#[bench]
fn u64_deserialize(b: &mut Bencher) {
    let random_u64_serialized: [u8; 8] = rand::random();

    b.iter(|| {
        from_bytes::<u64>(&random_u64_serialized, None).unwrap();
    });
}

#[bench]
fn f32_serialize(b: &mut Bencher) {
    let random_f32: f32 = rand::random();

    b.iter(|| {
        to_bytes(&random_f32).unwrap();
    });
}

#[bench]
fn f32_deserialize(b: &mut Bencher) {
    let random_f32_serialized: [u8; 4] = rand::random();

    b.iter(|| {
        from_bytes::<f32>(&random_f32_serialized, None).unwrap();
    });
}

#[bench]
fn f64_serialize(b: &mut Bencher) {
    let random_f64: f64 = rand::random();

    b.iter(|| {
        to_bytes(&random_f64).unwrap();
    });
}

#[bench]
fn f64_deserialize(b: &mut Bencher) {
    let random_f64_serialized: [u8; 8] = rand::random();

    b.iter(|| {
        from_bytes::<f64>(&random_f64_serialized, None).unwrap();
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

fn random_string<R: Rng>(rng: &mut R, min_words_count: usize, max_words_count: usize) -> String {
    let lipsum_words_count: usize = rng.gen_range(min_words_count, max_words_count);

    lipsum::lipsum(lipsum_words_count)
}

#[bench]
fn random_string_short_serialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), 0, 32);

    b.iter(|| {
        to_bytes(&random_string).unwrap();
    });
}

#[bench]
fn random_string_short_deserialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), 0, 32);
    let random_string_serialized = to_bytes(&random_string).unwrap();

    b.iter(|| {
        from_bytes::<String>(&random_string_serialized, None).unwrap();
    });
}

#[bench]
fn random_string_medium_serialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), 128, 512);

    b.iter(|| {
        to_bytes(&random_string).unwrap();
    });
}

#[bench]
fn random_string_medium_deserialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), 128, 512);
    let random_string_serialized = to_bytes(&random_string).unwrap();

    b.iter(|| {
        from_bytes::<String>(&random_string_serialized, None).unwrap();
    });
}

#[bench]
fn random_string_long_serialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), 16384, 65536);

    b.iter(|| {
        to_bytes(&random_string).unwrap();
    });
}

#[bench]
fn random_string_long_deserialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), 16384, 65536);
    let random_string_serialized = to_bytes(&random_string).unwrap();

    b.iter(|| {
        from_bytes::<String>(&random_string_serialized, None).unwrap();
    });
}


// STRUCTS

#[derive(Serialize, Deserialize, MtProtoIdentifiable)]
#[id = "0xd594ba98"]
struct Foo {
    bar: bool,
    s: String,
    group: (i16, u64, i8),
}

impl Rand for Foo {
    fn rand<R: Rng>(rng: &mut R) -> Foo {
        Foo {
            bar: rng.gen(),
            // Generate moderately long strings for reference
            s: random_string(rng, 2048, 4096),
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
