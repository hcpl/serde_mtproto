#![feature(test)]


extern crate rand;
extern crate test;
extern crate serde_mtproto;


use serde_mtproto::{to_bytes, from_bytes};
use test::Bencher;


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
