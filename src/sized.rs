//! `MtProtoSized` trait for any Rust data structure a predictable size of its MTProto binary
//! representation can be computed.

use std::cmp;
use std::collections::{HashMap, BTreeMap};
use std::hash::Hash;
use std::mem;

use error::{self, ErrorKind};
use utils::check_seq_len;


pub const BOOL_SIZE: usize = 4;
pub const INT_SIZE: usize = 4;
pub const LONG_SIZE: usize = 8;
pub const DOUBLE_SIZE: usize = 8;
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

impl<T1: MtProtoSized> MtProtoSized for (T1,) {
    fn get_size_hint(&self) -> error::Result<usize> {
        self.0.get_size_hint()
    }
}

impl<T1: MtProtoSized, T2: MtProtoSized> MtProtoSized for (T1, T2) {
    fn get_size_hint(&self) -> error::Result<usize> {
        Ok(self.0.get_size_hint()? + self.1.get_size_hint()?)
    }
}
