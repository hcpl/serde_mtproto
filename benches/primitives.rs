#![feature(test)]


#[cfg(feature = "extprim")]
extern crate extprim;
extern crate rand;
extern crate test;
extern crate serde_mtproto;


use serde_mtproto::{to_bytes, to_writer, from_bytes};
use test::Bencher;


macro_rules! bench_primitive {
    ($($ty:ty, $ser:ident => $ser_bytes_ty:ty, $de:ident;)*) => {
        $(
            #[bench]
            fn $ser(b: &mut Bencher) {
                let random_value: $ty = rand::random();
                let mut v: $ser_bytes_ty = Default::default();

                b.iter(|| {
                    to_writer(v.as_mut(), &random_value).unwrap();
                });
            }

            #[bench]
            fn $de(b: &mut Bencher) {
                let random_value: $ty = rand::random();
                let random_value_serialized = to_bytes(&random_value).unwrap();

                b.iter(|| {
                    from_bytes::<$ty>(&random_value_serialized, None).unwrap();
                });
            }
        )*
    };
}


bench_primitive! {
    bool, bool_serialize => [u8; 4], bool_deserialize;

    i8,    i8_serialize    => [u8; 4],  i8_deserialize;
    i16,   i16_serialize   => [u8; 4],  i16_deserialize;
    i32,   i32_serialize   => [u8; 4],  i32_deserialize;
    i64,   i64_serialize   => [u8; 8],  i64_deserialize;
    isize, isize_serialize => [u8; 16], isize_deserialize;    // 16 just to be safe

    u8,    u8_serialize    => [u8; 4],  u8_deserialize;
    u16,   u16_serialize   => [u8; 4],  u16_deserialize;
    u32,   u32_serialize   => [u8; 4],  u32_deserialize;
    u64,   u64_serialize   => [u8; 8],  u64_deserialize;
    usize, usize_serialize => [u8; 16], usize_deserialize;    // same here

    f32, f32_serialize => [u8; 8], f32_deserialize;
    f64, f64_serialize => [u8; 8], f64_deserialize;
}


#[cfg(feature = "extprim")]
bench_primitive! {
    ::extprim::i128::i128, i128_serialize => [u8; 16], i128_deserialize;
    ::extprim::u128::u128, u128_serialize => [u8; 16], u128_deserialize;
}
