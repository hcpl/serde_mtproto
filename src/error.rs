//! When serializing or deserializing MTProto goes wrong.

use std::fmt;

use serde::{ser, de};


error_chain! {
    foreign_links {
        Io(::std::io::Error) #[doc = "Wraps an `io::Error`"];
        FromUtf8(::std::string::FromUtf8Error) #[doc = "Wraps a `FromUtf8Error`"];
    }

    errors {
        /// An error during serialization.
        Ser(kind: SerErrorKind) {
            description("serialization error in serde_mtproto")
            display("serialization error in serde_mtproto: {}", kind)
        }

        /// An error during deserialization.
        De(kind: DeErrorKind) {
            description("deserialization error in serde_mtproto")
            display("deserialization error in serde_mtproto: {}", kind)
        }

        /// Error while casting an integer.
        IntegerCast(num: u64) {
            description("error while casting an integer")
            display("error while casting an integer: {}", num)
        }

        /// Error while casting a floating-point number.
        FloatCast(num: f64) {
            description("error while casting a floating-point number")
            display("error while casting a floating-point number: {}", num)
        }

        /// A string that cannot be serialized because it exceeds a certain length limit.
        StringTooLong(len: usize) {
            description("string is too long to serialize")
            display("string of length {} is too long to serialize", len)
        }

        /// A byte sequence that cannot be serialized because it exceeds a certain length limit.
        ByteSeqTooLong(len: usize) {
            description("byte sequence is too long to serialize")
            display("byte sequence of length {} is too long to serialize", len)
        }

        /// A sequence that cannot be serialized because it exceeds a certain length limit.
        SeqTooLong(len: usize) {
            description("sequence is too long to serialize")
            display("sequence of length {} is too long to serialize", len)
        }
    }
}


/// Serialization error kinds.
#[derive(Debug)]
pub enum SerErrorKind {
    /// A convenient variant for String.
    Msg(String),
    /// Excess elements found, stores the needed count.
    ExcessElements(u32),
    /// Cannot serialize maps with unknown length.
    MapsWithUnknownLengthUnsupported,
    /// Not enough elements, stores the actual and needed count.
    NotEnoughElements(u32, u32),
    /// Cannot serialize sequences with unknown length.
    SeqsWithUnknownLengthUnsupported,
    /// A string that cannot be serialized because it exceeds a certain length limit.
    StringTooLong(usize),
    /// This `serde` data format doesn't support several types in the Serde data model.
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
pub enum SerSerdeType {
    /// Single character type.
    Char,
    /// None value of Option type.
    None,
    /// Some value of Option type.
    Some,
    /// Unit `()` type.
    Unit,
}

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
    /// A convenient variant for String.
    Msg(String),
    /// This `serde` data format doesn't support several types in the Serde data model.
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
pub enum DeSerdeType {
    /// `serde_mtproto` doesn't support `*_any` hint.
    Any,
    /// Single character type.
    Char,
    /// Option type.
    Option,
    /// Unit `()` type.
    Unit,
    /// `serde_mtproto` doesn't support `*_ignored_any` hint.
    IgnoredAny,
}

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
