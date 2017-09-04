use num_traits::cast::{NumCast, cast};

use error::{self, ErrorKind, ResultExt};


pub fn safe_cast<T: NumCast + Copy, U: NumCast>(n: T) -> error::Result<U> {
    cast(n).ok_or_else(|| {
        let upcasted = cast::<T, u64>(n).unwrap();    // Shouldn't panic
        ErrorKind::IntegerCast(upcasted).into()
    })
}

pub fn check_seq_len(len: usize) -> error::Result<()> {
    safe_cast::<usize, u32>(len)
        .map(|_| ())
        .chain_err(|| ErrorKind::SeqTooLong(len))
}
