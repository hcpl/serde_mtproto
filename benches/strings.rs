#![feature(test)]


extern crate lipsum;
extern crate rand;
extern crate serde_mtproto;
extern crate test;


use rand::Rng;
use serde_mtproto::{MtProtoSized, to_bytes, to_writer, from_bytes};
use test::Bencher;


macro_rules! bench_string {
    ($( ($ser:ident, $de:ident) => $init_value:expr, )*) => {
        $(
            #[bench]
            fn $ser(b: &mut Bencher) {
                let string = $init_value;
                let mut v = vec![0; string.get_size_hint().unwrap()];

                b.iter(|| {
                    to_writer(v.as_mut_slice(), &string).unwrap();
                });
            }

            #[bench]
            fn $de(b: &mut Bencher) {
                let string_serialized = to_bytes(&$init_value).unwrap();

                b.iter(|| {
                    from_bytes::<String>(&string_serialized, None).unwrap();
                });
            }
        )*
    };
}


// STATIC STRINGS

bench_string! {
    (string_empty_serialize, string_empty_deserialize) => "",
    (string_short_serialize, string_short_deserialize) => "foobar",
    (string_medium_serialize, string_medium_deserialize) => lipsum::LOREM_IPSUM,
    (string_long_serialize, string_long_deserialize) => lipsum::LIBER_PRIMUS,
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

macro_rules! bench_random_string {
    ($( ($ser:ident, $de:ident) => $range:expr, )*) => {
        bench_string! {
            $( ($ser, $de) => random_string(&mut rand::thread_rng(), $range) )*
        }
    };
}

bench_random_string! {
    (random_string_short_serialize, random_string_short_deserialize) => RANGE_SHORT,
    (random_string_medium_serialize, random_string_medium_deserialize) => RANGE_MEDIUM,
    (random_string_long_serialize, random_string_long_deserialize) => RANGE_LONG,
    (random_string_very_long_serialize, random_string_very_long_deserialize) => RANGE_VERY_LONG,
}
