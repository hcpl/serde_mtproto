//! When serializing or deserializing MTProto goes wrong.

use std::fmt;

use serde::{ser, de};


error_chain! {
    foreign_links {
        Io(::std::io::Error);
        FromUtf8(::std::string::FromUtf8Error);
    }

    errors {
        Ser(kind: SerErrorKind) {
            description("serialization error in serde_mtproto")
            display("serialization error in serde_mtproto: {}", kind)
        }

        De(kind: DeErrorKind) {
            description("deserialization error in serde_mtproto")
            display("deserialization error in serde_mtproto: {}", kind)
        }

        IntegerCast {
            description("error while casting an integer")
            display("error while casting an integer")
        }

        StringTooLong(len: usize) {
            description("string is too long to serialize")
            display("string of length {} is too long to serialize", len)
        }

        SeqTooLong(len: usize) {
            description("sequence is too long to serialize")
            display("sequence of length {} is too long to serialize", len)
        }
    }
}


/// Serialization error kinds.
#[derive(Debug)]
pub enum SerErrorKind {
    Msg(String),
    ExcessElements(u32),
    MapsWithUnknownLengthUnsupported,
    NotEnoughElements(u32, u32),
    SeqsWithUnknownLengthUnsupported,
    StringTooLong(usize),
    UnsupportedSerdeType(SerSerdeType),
}

impl fmt::Display for SerErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SerErrorKind::Msg(ref string) => {
                write!(f, "custom string: {}", string)
            },
            SerErrorKind::ExcessElements(len) => {
                write!(f, "excess elements, need no more than {}", len)
            },
            SerErrorKind::MapsWithUnknownLengthUnsupported => {
                write!(f, "maps with ahead-of-time unknown length are not supported")
            },
            SerErrorKind::NotEnoughElements(unexpected_len, expected_len) => {
                write!(f, "not enough elements: have {}, need {}", unexpected_len, expected_len)
            },
            SerErrorKind::SeqsWithUnknownLengthUnsupported => {
                write!(f, "seqs with ahead-of-time unknown length are not supported")
            },
            SerErrorKind::StringTooLong(len) => {
                write!(f, "string of length {} is too long to serialize", len)
            },
            SerErrorKind::UnsupportedSerdeType(ref type_) => {
                write!(f, "{} type is not supported for serialization", type_)
            },
        }
    }
}

/// Serde serialization data types that are not supported by `serde_mtproto`.
#[derive(Debug)]
pub enum SerSerdeType { Char, None, Some, Unit }

impl fmt::Display for SerSerdeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repr = match *self {
            SerSerdeType::Char => "char",
            SerSerdeType::None => "none",
            SerSerdeType::Some => "some",
            SerSerdeType::Unit => "unit",
        };

        f.write_str(repr)
    }
}

impl From<SerErrorKind> for Error {
    fn from(kind: SerErrorKind) -> Error {
        ErrorKind::Ser(kind).into()
    }
}


/// Deserialization error kinds.
#[derive(Debug)]
pub enum DeErrorKind {
    Msg(String),
    UnsupportedSerdeType(DeSerdeType),
}

impl fmt::Display for DeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeErrorKind::Msg(ref string) => {
                write!(f, "custom string: {}", string)
            },
            DeErrorKind::UnsupportedSerdeType(ref type_) => {
                write!(f, "{} type is not supported for deserialization", type_)
            },
        }
    }
}

/// Serde deserialization data types that are not supported by `serde_mtproto`.
#[derive(Debug)]
pub enum DeSerdeType { Any, Char, Option, Unit, IgnoredAny }

impl fmt::Display for DeSerdeType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let repr = match *self {
            DeSerdeType::Any => "any",
            DeSerdeType::Char => "char",
            DeSerdeType::Option => "option",
            DeSerdeType::Unit => "unit",
            DeSerdeType::IgnoredAny => "ignored_any",
        };

        f.write_str(repr)
    }
}

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
