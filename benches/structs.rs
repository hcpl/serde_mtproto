#![feature(test)]


extern crate lipsum;
extern crate rand;
#[macro_use]
extern crate serde_derive;
extern crate serde_mtproto;
#[macro_use]
extern crate serde_mtproto_derive;
extern crate test;


use rand::{Rand, Rng};
use serde_mtproto::{to_bytes, from_bytes};
use test::Bencher;


fn random_string<R: Rng>(rng: &mut R, words_count: (usize, usize)) -> String {
    let lipsum_words_count: usize = rng.gen_range(words_count.0, words_count.1);

    lipsum::lipsum(lipsum_words_count)
}


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
            s: random_string(rng, (2048, 4096)),
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
