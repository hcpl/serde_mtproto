//! Testing inputs obtained from fuzzers.

extern crate serde_mtproto;

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
            #[should_panic]
            fn $roundtrip_fail() {
                const DATA: &[u8] = $data_fail;

                let (value, remaining) = serde_mtproto::from_bytes_reuse::<$ty_fail>(DATA, &[]).unwrap();
                let bytes = serde_mtproto::to_bytes(&value).unwrap();

                assert_eq!(bytes.len() + remaining.len(), DATA.len());
                assert_eq!(bytes, &DATA[0..bytes.len()]);
            }
        )+
    };
}

test_roundtrips! {
    success {
        roundtrip_success1 => b"\x00\x00\x00\x00\x00\x00\x00\x08" as String,
    }

    fail {
        roundtrip_fail1 => b"\x00\n\xff\xf8\xff\xfb\xff\xff\xff\xff\x07\x00\x00\x00\xfe\xff\xff\xff\xff\xff\xff" as String,
        roundtrip_fail2 => b"\xfe\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00" as String,
    }
}
