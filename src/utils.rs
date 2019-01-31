use num_traits::cast::cast;
use num_traits::float::Float;
use num_traits::int::PrimInt;
use num_traits::sign::{Signed, Unsigned};

use error::{self, ErrorKind, ResultExt};


pub(crate) type IntMax = i128;
pub(crate) type UIntMax = u128;


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


pub(crate) fn i128_from_parts(hi: i64, lo: u64) -> i128 {
    i128::from(hi) << 64 | i128::from(lo)
}

pub(crate) fn u128_from_parts(hi: u64, lo: u64) -> u128 {
    u128::from(hi) << 64 | u128::from(lo)
}

pub(crate) fn i128_to_parts(n: i128) -> (i64, u64) {
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::cast_sign_loss, clippy::cast_possible_truncation))]
    let lo = n as u64;
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::cast_possible_truncation))]
    let hi = (n >> 64) as i64;

    (hi, lo)
}

pub(crate) fn u128_to_parts(n: u128) -> (u64, u64) {
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::cast_possible_truncation))]
    let lo = n as u64;
    #[cfg_attr(feature = "cargo-clippy", allow(clippy::cast_possible_truncation))]
    let hi = (n >> 64) as u64;

    (hi, lo)
}


#[cfg(test)]
mod tests {
    const I128_PARTS: &[(i128, (i64, u64))] = &[
        ( 0x0000_0000_0000_0000_0000_0000_0000_0000, ( 0x0000_0000_0000_0000, 0x0000_0000_0000_0000)),
        ( 0x0000_0000_0000_0000_FFFF_FFFF_FFFF_FFFF, ( 0x0000_0000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF)),
        ( 0x7777_7777_7777_7777_FFFF_FFFF_FFFF_FFFF, ( 0x7777_7777_7777_7777, 0xFFFF_FFFF_FFFF_FFFF)),
        (-0x8000_0000_0000_0000_0000_0000_0000_0000, (-0x8000_0000_0000_0000, 0x0000_0000_0000_0000)),
        (-0x0000_0000_0000_0000_0000_0000_0000_0001, (-0x0000_0000_0000_0001, 0xFFFF_FFFF_FFFF_FFFF)),
    ];

    #[test]
    fn i128_from_parts() {
        for &(n, (hi, lo)) in I128_PARTS {
            assert_eq!(::utils::i128_from_parts(hi, lo), n);
        }
    }

    #[test]
    fn i128_to_parts() {
        for &(n, (hi, lo)) in I128_PARTS {
            assert_eq!(::utils::i128_to_parts(n), (hi, lo));
        }
    }


    const U128_PARTS: &[(u128, (u64, u64))] = &[
        (0x0000_0000_0000_0000_0000_0000_0000_0000, (0x0000_0000_0000_0000, 0x0000_0000_0000_0000)),
        (0x0000_0000_0000_0000_FFFF_FFFF_FFFF_FFFF, (0x0000_0000_0000_0000, 0xFFFF_FFFF_FFFF_FFFF)),
        (0x7777_7777_7777_7777_FFFF_FFFF_FFFF_FFFF, (0x7777_7777_7777_7777, 0xFFFF_FFFF_FFFF_FFFF)),
        (0x8000_0000_0000_0000_0000_0000_0000_0000, (0x8000_0000_0000_0000, 0x0000_0000_0000_0000)),
        (0xFFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF, (0xFFFF_FFFF_FFFF_FFFF, 0xFFFF_FFFF_FFFF_FFFF)),
    ];

    #[test]
    fn u128_from_parts() {
        for &(n, (hi, lo)) in U128_PARTS {
            assert_eq!(::utils::u128_from_parts(hi, lo), n);
        }
    }

    #[test]
    fn u128_to_parts() {
        for &(n, (hi, lo)) in U128_PARTS {
            assert_eq!(::utils::u128_to_parts(n), (hi, lo));
        }
    }

    #[cfg(feature = "quickcheck")]
    mod quickcheck {
        quickcheck! {
            // Quickcheck doesn't have an `Arbitrary` impl for `i128`/`u128`, so we need a
            // workaround.
            fn i128_parts_roundtrip(parts: (i64, u64)) -> bool {
                let (hi, lo) = parts;
                let n = ::utils::i128_from_parts(hi, lo);
                let (hi2, lo2) = ::utils::i128_to_parts(n);

                (hi, lo) == (hi2, lo2)
            }

            fn u128_parts_roundtrip(parts: (u64, u64)) -> bool {
                let (hi, lo) = parts;
                let n = ::utils::u128_from_parts(hi, lo);
                let (hi2, lo2) = ::utils::u128_to_parts(n);

                (hi, lo) == (hi2, lo2)
            }
        }
    }
}
