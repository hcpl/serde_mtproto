use num_traits::cast::{NumCast, cast};

use error::{self, ErrorKind};


pub const TRUE_ID: i32 = -1720552011;
pub const FALSE_ID: i32 = -1132882121;


pub fn safe_cast<T: NumCast, U: NumCast>(n: T) -> error::Result<U> {
    cast(n).ok_or(ErrorKind::IntegerCast.into())
}
