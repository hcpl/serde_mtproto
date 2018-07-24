//! Testing inputs obtained from fuzzers.

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_mtproto;
#[macro_use]
extern crate serde_mtproto_derive;

// Tests under `success` should successfully make a roundtrip for a prefix of
// `DATA` while those under `fail` should fail to parse and return `Err`. Tests
// are arranged this way to detect undesirable panics.
macro_rules! test_roundtrips {
    (
        success {
            $($roundtrip_success:ident => $data_success:tt as $ty_success:ty,)+
        }

        fail {
            $($roundtrip_fail:ident => $data_fail:tt as $ty_fail:ty,)+
        }
    ) => {
        $(
            #[test]
            fn $roundtrip_success() {
                const DATA: &[u8] = $data_success;

                let (value, remaining) = serde_mtproto::from_bytes_reuse::<$ty_success>(DATA, &[]).unwrap();
                let bytes = serde_mtproto::to_bytes(&value).unwrap();

                assert_eq!(bytes.len() + remaining.len(), DATA.len());
                assert_eq!(bytes, &DATA[0..bytes.len()]);
            }
        )+

        $(
            #[test]
            fn $roundtrip_fail() {
                const DATA: &[u8] = $data_fail;

                assert!(serde_mtproto::from_bytes::<$ty_fail>(DATA, &[]).is_err());
            }
        )+
    };
}


#[derive(Debug, Serialize, Deserialize, MtProtoIdentifiable, MtProtoSized)]
#[mtproto_identifiable(id = "0xdeadbeef")]
struct Foo {
    a: String,
    b: Vec<i16>,
    c: serde_mtproto::ByteBuf,
}

test_roundtrips! {
    success {
        roundtrip_success1 => b"\x00\x00\x00\x00\x00\x00\x00\x08" as String,
    }

    fail {
        roundtrip_fail1 => b"\x00\n\xff\xf8\xff\xfb\xff\xff\xff\xff\x07\x00\x00\x00\xfe\xff\xff\xff\xff\xff\xff" as String,
        roundtrip_fail2 => b"\xfe\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00" as String,
        roundtrip_fail3 => b"\x00\x00\x00\x00\x00\x10\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\xff\xff\xff\xff\xff\xff\xff\xff\xff\xfb\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\xff\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\n" as Foo,
    }
}
