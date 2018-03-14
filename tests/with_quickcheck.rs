#[macro_use]
extern crate quickcheck;
#[macro_use]
extern crate quickcheck_derive;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_mtproto;
#[macro_use]
extern crate serde_mtproto_derive;


use std::collections::BTreeMap;

use quickcheck::TestResult;
//use serde_mtproto::ByteBuf;
use serde_mtproto::{Boxed, BoxedWithSize, Identifiable, WithSize};


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Arbitrary, MtProtoIdentifiable, MtProtoSized)]
#[id = "0x02020202"]
struct PhantomStruct;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Arbitrary, MtProtoIdentifiable, MtProtoSized)]
#[id = "0xa821f7fe"]
struct SimpleStruct {
    field1: bool,
    field2: PhantomStruct,
    field3: String,
    field4: Boxed<SimpleEnum>,
    field5: SimpleStruct2,
    field6: usize,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Arbitrary, MtProtoIdentifiable, MtProtoSized)]
#[id = "0x341b1c93"]
struct SimpleStruct2((i8,), PhantomStruct, WithSize<(isize, u16)>);

#[cfg_attr(feature = "cargo-clippy", allow(enum_variant_names))]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, Arbitrary, MtProtoIdentifiable, MtProtoSized)]
enum SimpleEnum {
    #[id = "0x2d893c40"]
    Variant1,
    #[id = "0x037cb665"]
    Variant2,
    #[id = "0xb5a92a28"]
    Variant3 {
        variant3_field1: Vec<u8>, // replace by ByteBuf
        variant3_field2: BoxedWithSize<u32>, // replace by [u32; 4]
    },
    #[id = "0x8d72c9e1"]
    Variant4(((u64, i32), f32, BTreeMap<i8, String>, f64)), // replace <i8, String> by <i8, [String; 0]>
}


quickcheck! {
    fn ser_de_reversible(data: SimpleStruct) -> bool {
        println!("Received random data: {:?}", &data);
        let enum_variant_id = data.field4.inner().enum_variant_id().unwrap();
        let ser = serde_mtproto::to_bytes(&data).unwrap();
        println!("Serialized bytes: {:?}", &ser);
        let de = serde_mtproto::from_bytes::<SimpleStruct>(&ser, &[enum_variant_id]).unwrap();
        println!("Deserialized data: {:?}", &de);

        de == data
    }

    fn de_ser_reversible(byte_buf: Vec<u8>) -> TestResult {
        println!("Received random byte sequence: {:?}", &byte_buf);
        if let Ok(de) = serde_mtproto::from_bytes::<SimpleStruct>(&byte_buf, &[]) {
            println!("Deserialized data: {:?}", &de);
            let ser = serde_mtproto::to_bytes(&de).unwrap();
            println!("Serailized bytes: {:?}", &ser);

            TestResult::from_bool(ser == byte_buf)
        } else {
            TestResult::discard()
        }
    }
}
