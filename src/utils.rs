use num_traits::cast::{NumCast, cast};

use error::{self, ErrorKind};


pub fn safe_cast<T: NumCast, U: NumCast>(n: T) -> error::Result<U> {
    cast(n).ok_or(ErrorKind::IntegerCast.into())
}
