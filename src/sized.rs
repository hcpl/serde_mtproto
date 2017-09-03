//! `MtProtoSized` trait for any Rust data structure a predictable size of its MTProto binary
//! representation can be computed.

use std::cmp;
use std::collections::{HashMap, BTreeMap};
use std::hash::Hash;
use std::mem;

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
    fn get_size_hint(&self) -> error::Result<usize>;
}

impl<'a, T: MtProtoSized> MtProtoSized for &'a T {
    fn get_size_hint(&self) -> error::Result<usize> {
        (*self).get_size_hint()
    }
}

macro_rules! impl_mt_proto_sized_for_primitives {
    () => {};

    ($type:ty => $size:expr, $($rest:tt)*) => {
        impl MtProtoSized for $type {
            fn get_size_hint(&self) -> error::Result<usize> {
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

    ::extprim::i128::i128 => INT128_SIZE,
    ::extprim::u128::u128 => INT128_SIZE,
}

impl<'a> MtProtoSized for &'a str {
    fn get_size_hint(&self) -> error::Result<usize> {
        let len = self.len();

        let size = if len <= 253 {
            (len + 1) + (4 - (len + 1) % 4) % 4
        } else if len <= 0xff_ff_ff {
            len + (4 - len % 4) % 4
        } else {
            bail!(ErrorKind::StringTooLong(len));
        };

        Ok(size)
    }
}

impl MtProtoSized for String {
    fn get_size_hint(&self) -> error::Result<usize> {
        self.as_str().get_size_hint()
    }
}

impl<'a, T: MtProtoSized> MtProtoSized for &'a [T] {
    fn get_size_hint(&self) -> error::Result<usize> {
        // If len >= 2 ** 32, it's not serializable at all.
        check_seq_len(self.len())?;

        let mut result = 4;    // 4 for slice length

        for elem in self.iter() {
            result += elem.get_size_hint()?;
        }

        Ok(result)
    }
}

impl<T: MtProtoSized> MtProtoSized for Vec<T> {
    fn get_size_hint(&self) -> error::Result<usize> {
        self.as_slice().get_size_hint()
    }
}

impl<K, V> MtProtoSized for HashMap<K, V>
    where K: Eq + Hash + MtProtoSized,
          V: MtProtoSized,
{
    fn get_size_hint(&self) -> error::Result<usize> {
        // If len >= 2 ** 32, it's not serializable at all.
        check_seq_len(self.len())?;

        let mut result = 4;    // 4 for map length

        for (k, v) in self.iter() {
            result += k.get_size_hint()?;
            result += v.get_size_hint()?;
        }

        Ok(result)
    }
}

impl<K: MtProtoSized, V: MtProtoSized> MtProtoSized for BTreeMap<K, V> {
    fn get_size_hint(&self) -> error::Result<usize> {
        // If len >= 2 ** 32, it's not serializable at all.
        check_seq_len(self.len())?;

        let mut result = 4;    // 4 for map length

        for (k, v) in self.iter() {
            result += k.get_size_hint()?;
            result += v.get_size_hint()?;
        }

        Ok(result)
    }
}

impl MtProtoSized for () {
    fn get_size_hint(&self) -> error::Result<usize> {
        Ok(0)
    }
}

macro_rules! impl_mt_proto_sized_for_tuple {
    ($($ident:ident : $ty:ident ,)*) => {
        impl<$($ty: MtProtoSized),*> MtProtoSized for ($($ty,)*) {
            fn get_size_hint(&self) -> error::Result<usize> {
                let mut result = 0;
                let &($(ref $ident,)*) = self;
                $( result += $ident.get_size_hint()?; )*
                Ok(result)
            }
        }
    }
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
