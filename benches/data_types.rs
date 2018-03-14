#![feature(test)]


extern crate lipsum;
extern crate rand;
#[macro_use]
extern crate rand_derive;
#[macro_use]
extern crate serde_derive;
extern crate serde_mtproto;
#[macro_use]
extern crate serde_mtproto_derive;
extern crate test;


use rand::{Rand, Rng};
use serde_mtproto::{MtProtoSized, to_bytes, to_writer, from_bytes};
use test::Bencher;


fn random_string<R: Rng>(rng: &mut R, words_count: (usize, usize)) -> String {
    let lipsum_words_count: usize = rng.gen_range(words_count.0, words_count.1);

    lipsum::lipsum(lipsum_words_count)
}


#[derive(Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[id = "0xd594ba98"]
struct Struct {
    bar: bool,
    s: String,
    group: (i16, u64, i8),
}

impl Rand for Struct {
    fn rand<R: Rng>(rng: &mut R) -> Struct {
        Struct {
            bar: rng.gen(),
            // Generate moderately long strings for reference
            s: random_string(rng, (2048, 4096)),
            group: rng.gen(),
        }
    }
}

#[bench]
fn struct_serialize(b: &mut Bencher) {
    let struct_ = Struct {
        bar: false,
        s: "Hello, world!".to_owned(),
        group: (-500, 0xffff_ffff_ffff, -64),
    };
    let mut v = vec![0; struct_.size_hint().unwrap()];

    b.iter(|| {
        to_writer(v.as_mut_slice(), &struct_).unwrap();
    });
}

#[bench]
fn struct_deserialize(b: &mut Bencher) {
    let struct_serialized = [
        55, 151, 121, 188,
        13, 72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33, 0, 0,
        12, 254, 255, 255,
        255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 255, 0, 0, 0, 0,
        192, 255, 255, 255,
    ];

    b.iter(|| {
        from_bytes::<Struct>(&struct_serialized, &[]).unwrap();
    });
}

#[bench]
fn random_struct_serialize(b: &mut Bencher) {
    let random_struct: Struct = rand::random();
    let mut v = vec![0; random_struct.size_hint().unwrap()];

    b.iter(|| {
        to_writer(v.as_mut_slice(), &random_struct).unwrap();
    });
}

#[bench]
fn random_struct_deserialize(b: &mut Bencher) {
    let random_struct: Struct = rand::random();
    let random_struct_serialized = to_bytes(&random_struct).unwrap();

    b.iter(|| {
        from_bytes::<Struct>(&random_struct_serialized, &[]).unwrap();
    });
}


#[derive(Rand, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[id = "0x200c5e59"]
struct Nothing;

#[bench]
fn nothing_serialize(b: &mut Bencher) {
    let nothing = Nothing;
    let mut v = vec![0; nothing.size_hint().unwrap()];

    b.iter(|| {
        to_writer(v.as_mut_slice(), &nothing).unwrap();
    });
}

#[bench]
fn nothing_deserialize(b: &mut Bencher) {
    let nothing_serialized = [];

    b.iter(|| {
        from_bytes::<Nothing>(&nothing_serialized, &[]).unwrap();
    });
}

#[bench]
fn random_nothing_serialize(b: &mut Bencher) {
    let random_nothing: Nothing = rand::random();
    let mut v = vec![0; random_nothing.size_hint().unwrap()];

    b.iter(|| {
        to_writer(v.as_mut_slice(), &random_nothing).unwrap();
    });
}

#[bench]
fn random_nothing_deserialize(b: &mut Bencher) {
    let random_nothing: Nothing = rand::random();
    let random_nothing_serialized = to_bytes(&random_nothing).unwrap();

    b.iter(|| {
        from_bytes::<Nothing>(&random_nothing_serialized, &[]).unwrap();
    });
}
