use num_traits::cast::cast;
use num_traits::float::Float;
use num_traits::int::PrimInt;
use num_traits::sign::Unsigned;

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

pub fn safe_uint_eq<T, U>(x: T, y: U) -> bool
    where T: PrimInt + Unsigned,
          U: PrimInt + Unsigned,
{
    if let Some(ux) = cast::<T, U>(x) { // check if T \subseteq U ...
        ux == y
    } else if let Some(ty) = cast::<U, T>(y) { // check above failed, then it must be U \subseteq T here
        x == ty
    } else {
        unreachable!("This kind of comparison always involves upcasting the narrower number \
                      to the wider representation since at least one of T \\subseteq U or \
                      U \\subseteq T must be true");
    }
}
