#[macro_use]
extern crate quickcheck;
#[macro_use]
extern crate quickcheck_derive;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_mtproto;
#[macro_use]
extern crate serde_mtproto_derive;


use std::collections::BTreeMap;

use quickcheck::TestResult;
use rand::Rng;
//use serde_mtproto::ByteBuf;
use serde_mtproto::{Boxed, Identifiable, WithSize};


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Arbitrary, MtProtoIdentifiable, MtProtoSized)]
#[mtproto_identifiable(id = "0x02020202")]
struct PhantomStruct;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Arbitrary, MtProtoIdentifiable, MtProtoSized)]
#[mtproto_identifiable(id = "0xa821f7fe")]
struct SimpleStruct {
    field1: bool,
    field2: PhantomStruct,
    field3: String,
    field4: Boxed<SimpleEnum>,
    field5: SimpleStruct2,
    field6: i16,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Arbitrary, MtProtoIdentifiable, MtProtoSized)]
#[mtproto_identifiable(id = "0x341b1c93")]
struct SimpleStruct2((i8,), PhantomStruct, WithSize<(i64, u16)>);

#[cfg_attr(feature = "cargo-clippy", allow(enum_variant_names))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Arbitrary, MtProtoIdentifiable, MtProtoSized)]
enum SimpleEnum {
    #[mtproto_identifiable(id = "0x2d893c40")]
    Variant1,
    #[mtproto_identifiable(id = "0x037cb665")]
    Variant2,
    #[mtproto_identifiable(id = "0xb5a92a28")]
    Variant3 {
        variant3_field1: Vec<u8>, // replace by ByteBuf
        variant3_field2: Boxed<WithSize<u32>>, // replace by [u32; 4]
    },
    #[mtproto_identifiable(id = "0x8d72c9e1")]
    Variant4(((u64, i32), f32, BTreeMap<i8, String>, f64)), // replace <i8, String> by <i8, [String; 0]>
}


quickcheck! {
    fn ser_de_reversible(data: SimpleStruct) -> bool {
        let enum_variant_id = data.field4.inner().enum_variant_id().unwrap();

        let ser = serde_mtproto::to_bytes(&data).unwrap();
        let de = serde_mtproto::from_bytes::<SimpleStruct>(&ser, &[enum_variant_id]).unwrap();

        de == data
    }

    fn de_ser_reversible(byte_buf: Vec<u8>) -> TestResult {
        if let Ok(de) = serde_mtproto::from_bytes::<SimpleStruct>(&byte_buf, &[]) {
            let ser = serde_mtproto::to_bytes(&de).unwrap();

            TestResult::from_bool(ser == byte_buf)
        } else {
            TestResult::discard()
        }
    }
}
