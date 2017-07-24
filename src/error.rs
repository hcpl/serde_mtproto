use std::fmt;

use serde::{ser, de};


error_chain! {
    foreign_links {
        Io(::std::io::Error);
    }
}

impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        ErrorKind::Msg(msg.to_string()).into()
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        ErrorKind::Msg(msg.to_string()).into()
    }
}
