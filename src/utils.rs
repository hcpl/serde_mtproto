use num_traits::cast::{NumCast, cast};

use error::{self, ErrorKind, ResultExt};


pub fn safe_cast<T: NumCast, U: NumCast>(n: T) -> error::Result<U> {
    cast(n).ok_or(ErrorKind::IntegerCast.into())
}

pub fn check_seq_len(len: usize) -> error::Result<()> {
    safe_cast::<usize, u32>(len)
        .map(|_| ())
        .chain_err(|| ErrorKind::SeqTooLong(len))
}
