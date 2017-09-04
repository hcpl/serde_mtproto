#![feature(test)]


#[cfg(feature = "extprim")]
extern crate extprim;
extern crate rand;
extern crate test;
extern crate serde_mtproto;


use serde_mtproto::{to_bytes, from_bytes};
use test::Bencher;


macro_rules! bench_with_ser {
    ($ty:ty, $ser:ident, $de:ident) => {
        #[bench]
        fn $ser(b: &mut Bencher) {
            let random_value: $ty = rand::random();

            b.iter(|| {
                to_bytes(&random_value).unwrap();
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
    };
}

bench_with_ser!(bool, bool_serialize, bool_deserialize);
bench_with_ser!(isize, isize_serialize, isize_deserialize);
bench_with_ser!(usize, usize_serialize, usize_deserialize);


macro_rules! fixed_size_bench {
    ($ty:ty, $ser:ident, $de:ident => $de_bytes_ty:ty: $de_init:expr) => {
        #[bench]
        fn $ser(b: &mut Bencher) {
            let random_num: $ty = rand::random();

            b.iter(|| {
                to_bytes(&random_num).unwrap();
            });
        }

        #[bench]
        fn $de(b: &mut Bencher) {
            let random_num_serialized: $de_bytes_ty = $de_init;

            b.iter(|| {
                from_bytes::<$ty>(&random_num_serialized, None).unwrap();
            });
        }
    };
}

fixed_size_bench!(i8, i8_serialize, i8_deserialize => [u8; 4]: [rand::random(), 0, 0, 0]);
fixed_size_bench!(i16, i16_serialize, i16_deserialize => [u8; 4]: [rand::random(), rand::random(), 0, 0]);
fixed_size_bench!(i32, i32_serialize, i32_deserialize => [u8; 4]: rand::random());
fixed_size_bench!(i64, i64_serialize, i64_deserialize => [u8; 8]: rand::random());

fixed_size_bench!(u8, u8_serialize, u8_deserialize => [u8; 4]: [rand::random(), 0, 0, 0]);
fixed_size_bench!(u16, u16_serialize, u16_deserialize => [u8; 4]: [rand::random(), rand::random(), 0, 0]);
fixed_size_bench!(u32, u32_serialize, u32_deserialize => [u8; 4]: rand::random());
fixed_size_bench!(u64, u64_serialize, u64_deserialize => [u8; 8]: rand::random());

fixed_size_bench!(f32, f32_serialize, f32_deserialize => [u8; 4]: rand::random());
fixed_size_bench!(f64, f64_serialize, f64_deserialize => [u8; 8]: rand::random());

#[cfg(feature = "extprim")]
fixed_size_bench!(::extprim::i128::i128, i128_serialize, i128_deserialize => [u8; 16]: rand::random());
#[cfg(feature = "extprim")]
fixed_size_bench!(::extprim::u128::u128, u128_serialize, u128_deserialize => [u8; 16]: rand::random());
