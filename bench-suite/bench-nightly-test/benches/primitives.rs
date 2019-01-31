#![feature(test)]


extern crate rand;
extern crate test;
extern crate serde_mtproto;


use serde_mtproto::{to_bytes, to_writer, from_bytes};
use test::Bencher;


macro_rules! bench_primitive {
    ($($ty:ty, $ser:ident => [u8; $ser_bytes_count:expr], $de:ident;)*) => {
        $(
            #[bench]
            fn $ser(b: &mut Bencher) {
                let random_value: $ty = rand::random();
                let mut v: [u8; $ser_bytes_count] = [0; $ser_bytes_count];

                b.iter(|| {
                    to_writer(v.as_mut(), &random_value).unwrap();
                });
            }

            #[bench]
            fn $de(b: &mut Bencher) {
                let random_value: $ty = rand::random();
                let random_value_serialized = to_bytes(&random_value).unwrap();

                b.iter(|| {
                    from_bytes::<$ty>(&random_value_serialized, &[]).unwrap();
                });
            }
        )*
    };
}


bench_primitive! {
    bool, bool_serialize => [u8; 4], bool_deserialize;

    i8,   i8_serialize   => [u8; 4], i8_deserialize;
    i16,  i16_serialize  => [u8; 4], i16_deserialize;
    i32,  i32_serialize  => [u8; 4], i32_deserialize;
    i64,  i64_serialize  => [u8; 8], i64_deserialize;
    i128, i128_serialize => [u8; 16], i128_deserialize;

    u8,   u8_serialize   => [u8; 4], u8_deserialize;
    u16,  u16_serialize  => [u8; 4], u16_deserialize;
    u32,  u32_serialize  => [u8; 4], u32_deserialize;
    u64,  u64_serialize  => [u8; 8], u64_deserialize;
    u128, u128_serialize => [u8; 16], u128_deserialize;

    f32, f32_serialize => [u8; 8], f32_deserialize;
    f64, f64_serialize => [u8; 8], f64_deserialize;

    (i128, i128), two_i128_tuple_serialize => [u8; 32], two_i128_tuple_deserialize;
    (u128, u128), two_u128_tuple_serialize => [u8; 32], two_u128_tuple_deserialize;

    // randomly shuffled
    (u8, i16, f32, f64, i64, u64, u32, i32, i8, u16),    // <- truly random!
        all_numeric_primitives_tuple_serialize => [u8; 56], all_numeric_primitives_tuple_deserialize;

    [i8; 32],   i8_array32_serialize   => [u8; 128], i8_array32_deserialize;
    [i16; 32],  i16_array32_serialize  => [u8; 128], i16_array32_deserialize;
    [i32; 32],  i32_array32_serialize  => [u8; 128], i32_array32_deserialize;
    [i64; 32],  i64_array32_serialize  => [u8; 256], i64_array32_deserialize;
    [i128; 32], i128_array32_serialize => [u8; 512], i128_array32_deserialize;

    [u8; 32],   u8_array32_serialize   => [u8; 128], u8_array32_deserialize;
    [u16; 32],  u16_array32_serialize  => [u8; 128], u16_array32_deserialize;
    [u32; 32],  u32_array32_serialize  => [u8; 128], u32_array32_deserialize;
    [u64; 32],  u64_array32_serialize  => [u8; 256], u64_array32_deserialize;
    [u128; 32], u128_array32_serialize => [u8; 512], u128_array32_deserialize;

    [f32; 32], f32_array32_serialize => [u8; 256], f32_array32_deserialize;
    [f64; 32], f64_array32_serialize => [u8; 256], f64_array32_deserialize;
}
