#![feature(test)]


extern crate lipsum;
extern crate rand;
extern crate serde_mtproto;
extern crate test;


use rand::Rng;
use serde_mtproto::{to_bytes, from_bytes};
use test::Bencher;


// STATIC STRINGS

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


// RANDOM STRINGS

fn random_string<R: Rng>(rng: &mut R, words_count: (usize, usize)) -> String {
    let lipsum_words_count: usize = rng.gen_range(words_count.0, words_count.1);

    lipsum::lipsum(lipsum_words_count)
}


const RANGE_SHORT: (usize, usize) = (0, 32);
const RANGE_MEDIUM: (usize, usize) = (128, 512);
const RANGE_LONG: (usize, usize) = (32768, 65536);
const RANGE_VERY_LONG: (usize, usize) = (1048576, 2097152);

#[bench]
fn random_string_short_serialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), RANGE_SHORT);

    b.iter(|| {
        to_bytes(&random_string).unwrap();
    });
}

#[bench]
fn random_string_short_deserialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), RANGE_SHORT);
    let random_string_serialized = to_bytes(&random_string).unwrap();

    b.iter(|| {
        from_bytes::<String>(&random_string_serialized, None).unwrap();
    });
}

#[bench]
fn random_string_medium_serialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), RANGE_MEDIUM);

    b.iter(|| {
        to_bytes(&random_string).unwrap();
    });
}

#[bench]
fn random_string_medium_deserialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), RANGE_MEDIUM);
    let random_string_serialized = to_bytes(&random_string).unwrap();

    b.iter(|| {
        from_bytes::<String>(&random_string_serialized, None).unwrap();
    });
}

#[bench]
fn random_string_long_serialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), RANGE_LONG);

    b.iter(|| {
        to_bytes(&random_string).unwrap();
    });
}

#[bench]
fn random_string_long_deserialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), RANGE_LONG);
    let random_string_serialized = to_bytes(&random_string).unwrap();

    b.iter(|| {
        from_bytes::<String>(&random_string_serialized, None).unwrap();
    });
}

#[bench]
fn random_string_very_long_serialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), RANGE_VERY_LONG);

    b.iter(|| {
        to_bytes(&random_string).unwrap();
    });
}

#[bench]
fn random_string_very_long_deserialize(b: &mut Bencher) {
    let random_string = random_string(&mut rand::thread_rng(), RANGE_VERY_LONG);
    let random_string_serialized = to_bytes(&random_string).unwrap();

    b.iter(|| {
        from_bytes::<String>(&random_string_serialized, None).unwrap();
    });
}
