use num_traits::cast::cast;
use num_traits::float::Float;
use num_traits::int::PrimInt;

use error::{self, ErrorKind, ResultExt};


pub fn safe_int_cast<T: PrimInt + Copy, U: PrimInt>(n: T) -> error::Result<U> {
    cast(n).ok_or_else(|| {
        let upcasted = cast::<T, u64>(n).unwrap();    // Shouldn't panic
        ErrorKind::IntegerCast(upcasted).into()
    })
}

pub fn safe_float_cast<T: Float + Copy, U: Float>(n: T) -> error::Result<U> {
    cast(n).ok_or_else(|| {
        let upcasted = cast::<T, f64>(n).unwrap();    // Shouldn't panic
        ErrorKind::FloatCast(upcasted).into()
    })
}

pub fn check_seq_len(len: usize) -> error::Result<()> {
    safe_int_cast::<usize, u32>(len)
        .map(|_| ())
        .chain_err(|| ErrorKind::SeqTooLong(len))
}
