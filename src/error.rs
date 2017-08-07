use std::fmt;

use serde::{ser, de};


error_chain! {
    foreign_links {
        Io(::std::io::Error);
        FromUtf8(::std::string::FromUtf8Error);
    }

    errors {
        Ser(kind: SerErrorKind)
        De(kind: DeErrorKind)
    }
}


#[derive(Debug)]
pub enum SerErrorKind {
    Msg(String),
    ExcessElements(usize),
    IntegerOverflowingCast,
    SeqWithUnknownLengthUnsupported,
    UnsupportedSerdeType(SerSerdeType),
}

#[derive(Debug)]
pub enum SerSerdeType { Char, None, Some, Unit, Map }

impl From<SerErrorKind> for Error {
    fn from(kind: SerErrorKind) -> Error {
        ErrorKind::Ser(kind).into()
    }
}


#[derive(Debug)]
pub enum DeErrorKind {
    Msg(String),
    ExpectedBool,
    IntegerOverflowingCast,
    InvalidStrFirstByte255,
    UnsupportedSerdeType(DeSerdeType),
}

#[derive(Debug)]
pub enum DeSerdeType { Any, Char, Option, Unit, Map, IgnoredAny }

impl From<DeErrorKind> for Error {
    fn from(kind: DeErrorKind) -> Error {
        ErrorKind::De(kind).into()
    }
}


impl ser::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        SerErrorKind::Msg(msg.to_string()).into()
    }
}

impl de::Error for Error {
    fn custom<T: fmt::Display>(msg: T) -> Error {
        DeErrorKind::Msg(msg.to_string()).into()
    }
}
