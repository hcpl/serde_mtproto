use num_traits::cast::cast;
use num_traits::float::Float;
use num_traits::int::PrimInt;
use num_traits::sign::{Signed, Unsigned};

use error::{self, ErrorKind, ResultExt};


#[cfg(not(stable_i128))] pub(crate) type IntMax = i64;
#[cfg(stable_i128)]      pub(crate) type IntMax = i128;

#[cfg(not(stable_i128))] pub(crate) type UIntMax = u64;
#[cfg(stable_i128)]      pub(crate) type UIntMax = u128;


pub(crate) fn safe_int_cast<T, U>(n: T) -> error::Result<U>
    where T: PrimInt + Signed,
          U: PrimInt + Signed,
{
    cast(n).ok_or_else(|| {
        let upcasted = cast::<T, IntMax>(n).unwrap();    // Shouldn't panic
        ErrorKind::SignedIntegerCast(upcasted).into()
    })
}

pub(crate) fn safe_uint_cast<T, U>(n: T) -> error::Result<U>
    where T: PrimInt + Unsigned,
          U: PrimInt + Unsigned,
{
    cast(n).ok_or_else(|| {
        let upcasted = cast::<T, UIntMax>(n).unwrap();    // Shouldn't panic
        ErrorKind::UnsignedIntegerCast(upcasted).into()
    })
}

pub(crate) fn safe_float_cast<T: Float + Copy, U: Float>(n: T) -> error::Result<U> {
    cast(n).ok_or_else(|| {
        let upcasted = cast::<T, f64>(n).unwrap();    // Shouldn't panic
        ErrorKind::FloatCast(upcasted).into()
    })
}

pub(crate) fn check_seq_len(len: usize) -> error::Result<()> {
    safe_uint_cast::<usize, u32>(len)
        .map(|_| ())
        .chain_err(|| ErrorKind::SeqTooLong(len))
}

pub(crate) fn safe_uint_eq<T, U>(x: T, y: U) -> bool
    where T: PrimInt + Unsigned,
          U: PrimInt + Unsigned,
{
    if let Some(ux) = cast::<T, U>(x) { // check if T \subseteq U ...
        ux == y
    } else if let Some(ty) = cast::<U, T>(y) { // check above failed, then it must be U \subset T here
        x == ty
    } else {
        unreachable!("This kind of comparison always involves upcasting the narrower number \
                      to the wider representation since at least one of T \\subseteq U or \
                      U \\subseteq T must be true");
    }
}
