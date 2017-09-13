//! `MtProtoSized` trait for any Rust data structure a predictable size of its MTProto binary
//! representation can be computed.
//!
//! # Examples
//!
//! ```
//! use serde_mtproto::{MtProtoSized, ByteBuf};
//!
//! struct Something {
//!     name: String,
//!     small_num: u16,
//!     raw_data: ByteBuf,
//!     pair: (i8, u64),
//! }
//!
//! // Implement manually
//!
//! impl MtProtoSized for Something {
//!     fn size_hint(&self) -> serde_mtproto::Result<usize> {
//!         let mut result = 0;
//!
//!         result += self.name.size_hint()?;
//!         result += self.small_num.size_hint()?;
//!         result += self.raw_data.size_hint()?;
//!         result += self.pair.size_hint()?;
//!
//!         Ok(result)
//!     }
//! }
//!
//! # fn run() -> serde_mtproto::Result<()> {
//! let smth = Something {
//!     name: "John Smith".to_owned(),
//!     small_num: 2000u16,
//!     raw_data: ByteBuf::from(vec![0xf4, 0x58, 0x2e, 0x33]),
//!     pair: (-50, 0xffff_ffff_ffff_ffff),
//! };
//!
//! // "John Smith" => 1 byte length, 10 bytes data, 1 byte padding;
//! // 2000u16 => 4 bytes;
//! // ByteBuf { ... } => 1 byte length, 4 bytes data, 3 bytes padding;
//! // (-50, 0xffff_ffff_ffff_ffff) => 4 bytes + 8 bytes == 12 bytes;
//! //
//! // Total: 12 + 4 + 8 + 12 == 36 bytes
//!
//! assert_eq!(36, smth.size_hint()?);
//! #     Ok(())
//! # }
//!
//! # fn main() { run().unwrap(); }
//! ```
//!
//! Alternatively, `MtProtoSized` can be `#[derive]`d:
//!
//! ```
//! #[macro_use]
//! extern crate serde_mtproto_derive;
//!
//! #[derive(MtProtoSized)]
//! struct Something {
//!     name: String,
//!     small_num: u16,
//!     raw_data: Vec<u8>,
//!     pair: (i8, u64),
//! }
//!
//! # fn main() {}
//! ```
//!
//! The derived implementation is the same as the one shown above.

use std::cmp;
use std::collections::{HashMap, BTreeMap};
use std::hash::Hash;
use std::mem;

use serde_bytes::{ByteBuf, Bytes};

use error::{self, ErrorKind};
use utils::check_seq_len;


/// Size of a bool MtProto value.
pub const BOOL_SIZE: usize = 4;
/// Size of an int MtProto value.
pub const INT_SIZE: usize = 4;
/// Size of a long MtProto value.
pub const LONG_SIZE: usize = 8;
/// Size of a dobule MtProto value.
pub const DOUBLE_SIZE: usize = 8;
/// Size of an int128 MtProto value.
pub const INT128_SIZE: usize = 16;


/// A trait for a Rust data structure a predictable size of its MTProto binary representation
/// can be computed.
pub trait MtProtoSized {
    /// Compute the size of MTProto binary representation of this value without actually
    /// serializing it.
    ///
    /// Returns an `error::Result` because not any value can be serialized (e.g. strings and
    /// sequences that are too long).
    fn size_hint(&self) -> error::Result<usize>;
}


impl<'a, T: MtProtoSized> MtProtoSized for &'a T {
    fn size_hint(&self) -> error::Result<usize> {
        (*self).size_hint()
    }
}

macro_rules! impl_mt_proto_sized_for_primitives {
    () => {};

    ($type:ty => $size:expr, $($rest:tt)*) => {
        impl MtProtoSized for $type {
            fn size_hint(&self) -> error::Result<usize> {
                Ok($size)
            }
        }

        impl_mt_proto_sized_for_primitives! { $($rest)* }
    };
}

impl_mt_proto_sized_for_primitives! {
    bool => BOOL_SIZE,

    isize => cmp::max(mem::size_of::<isize>(), INT_SIZE),
    i8    => INT_SIZE,
    i16   => INT_SIZE,
    i32   => INT_SIZE,
    i64   => LONG_SIZE,

    usize => cmp::max(mem::size_of::<usize>(), INT_SIZE),
    u8    => INT_SIZE,
    u16   => INT_SIZE,
    u32   => INT_SIZE,
    u64   => LONG_SIZE,

    f32 => DOUBLE_SIZE,
    f64 => DOUBLE_SIZE,
}

#[cfg(feature = "extprim")]
impl_mt_proto_sized_for_primitives! {
    ::extprim::i128::i128 => INT128_SIZE,
    ::extprim::u128::u128 => INT128_SIZE,
}


/// Helper function for everything natually representable as a byte sequence.
fn byte_seq_size_hint(b: &[u8]) -> error::Result<usize> {
    let len = b.len();

    let (len_info, data, padding) = if len <= 253 {
        (1, len, (4 - (len + 1) % 4) % 4)
    } else if len <= 0xff_ff_ff {
        (4, len, (4 - len % 4) % 4)
    } else {
        bail!(ErrorKind::StringTooLong(len));
    };

    let size = len_info + data + padding;
    assert!(size % 4 == 0);

    Ok(size)
}

impl<'a> MtProtoSized for &'a str {
    fn size_hint(&self) -> error::Result<usize> {
        byte_seq_size_hint(self.as_bytes())
    }
}

impl MtProtoSized for String {
    fn size_hint(&self) -> error::Result<usize> {
        byte_seq_size_hint(self.as_bytes())
    }
}

impl<'a, T: MtProtoSized> MtProtoSized for &'a [T] {
    fn size_hint(&self) -> error::Result<usize> {
        // If len >= 2 ** 32, it's not serializable at all.
        check_seq_len(self.len())?;

        let mut result = 4;    // 4 for slice length

        for elem in self.iter() {
            result += elem.size_hint()?;
        }

        Ok(result)
    }
}

impl<T: MtProtoSized> MtProtoSized for Vec<T> {
    fn size_hint(&self) -> error::Result<usize> {
        self.as_slice().size_hint()
    }
}

impl<K, V> MtProtoSized for HashMap<K, V>
    where K: Eq + Hash + MtProtoSized,
          V: MtProtoSized,
{
    fn size_hint(&self) -> error::Result<usize> {
        // If len >= 2 ** 32, it's not serializable at all.
        check_seq_len(self.len())?;

        let mut result = 4;    // 4 for map length

        for (k, v) in self.iter() {
            result += k.size_hint()?;
            result += v.size_hint()?;
        }

        Ok(result)
    }
}

impl<K: MtProtoSized, V: MtProtoSized> MtProtoSized for BTreeMap<K, V> {
    fn size_hint(&self) -> error::Result<usize> {
        // If len >= 2 ** 32, it's not serializable at all.
        check_seq_len(self.len())?;

        let mut result = 4;    // 4 for map length

        for (k, v) in self.iter() {
            result += k.size_hint()?;
            result += v.size_hint()?;
        }

        Ok(result)
    }
}

impl MtProtoSized for () {
    fn size_hint(&self) -> error::Result<usize> {
        Ok(0)
    }
}

impl<'a> MtProtoSized for Bytes<'a> {
    fn size_hint(&self) -> error::Result<usize> {
        byte_seq_size_hint(self)
    }
}

impl MtProtoSized for ByteBuf {
    fn size_hint(&self) -> error::Result<usize> {
        byte_seq_size_hint(self)
    }
}

macro_rules! impl_mt_proto_sized_for_tuple {
    ($($ident:ident : $ty:ident ,)*) => {
        impl<$($ty: MtProtoSized),*> MtProtoSized for ($($ty,)*) {
            fn size_hint(&self) -> error::Result<usize> {
                let mut result = 0;
                let &($(ref $ident,)*) = self;
                $( result += $ident.size_hint()?; )*
                Ok(result)
            }
        }
    };
}

impl_mt_proto_sized_for_tuple! { x1: T1, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, x3: T3, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, x3: T3, x4: T4, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, x3: T3, x4: T4, x5: T5, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, x3: T3, x4: T4, x5: T5, x6: T6, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, x3: T3, x4: T4, x5: T5, x6: T6, x7: T7, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, x3: T3, x4: T4, x5: T5, x6: T6, x7: T7, x8: T8, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, x3: T3, x4: T4, x5: T5, x6: T6, x7: T7, x8: T8,
                                 x9: T9, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, x3: T3, x4: T4, x5: T5, x6: T6, x7: T7, x8: T8,
                                 x9: T9, x10: T10, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, x3: T3, x4: T4, x5: T5, x6: T6, x7: T7, x8: T8,
                                 x9: T9, x10: T10, x11: T11, }
impl_mt_proto_sized_for_tuple! { x1: T1, x2: T2, x3: T3, x4: T4, x5: T5, x6: T6, x7: T7, x8: T8,
                                 x9: T9, x10: T10, x11: T11, x12: T12, }

macro_rules! impl_mt_proto_sized_for_arrays {
    (0) => {
        impl<T> MtProtoSized for [T; 0] {
            fn size_hint(&self) -> error::Result<usize> {
                Ok(0)
            }
        }
    };
    ($size:expr) => {
        impl<T: MtProtoSized> MtProtoSized for [T; $size] {
            fn size_hint(&self) -> error::Result<usize> {
                let mut result = 0;

                for elem in self {
                    result += elem.size_hint()?;
                }

                Ok(result)
            }
        }
    };
    ($size:expr, $($rest:tt)*) => {
        impl_mt_proto_sized_for_arrays!($size);
        impl_mt_proto_sized_for_arrays!($($rest)*);
    };
}

impl_mt_proto_sized_for_arrays!(0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18,
                                19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 32);
