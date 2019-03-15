//! When serializing or deserializing MTProto goes wrong.

// Temporary fix for `std::error::Error::cause()` usage in `error_chain!`-generated code
// Should be resolved upstream in <https://github.com/rust-lang-nursery/error-chain/pull/255>
#![allow(deprecated)]

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

        /// Error while casting a signed integer.
        SignedIntegerCast(num: ::utils::IntMax) {
            description("error while casting a signed integer")
            display("error while casting a signed integer: {}", num)
        }

        /// Error while casting an unsigned integer.
        UnsignedIntegerCast(num: ::utils::UIntMax) {
            description("error while casting an unsigned integer")
            display("error while casting an unsigned integer: {}", num)
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
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum DeErrorKind {
    /// A convenient variant for String.
    Msg(String),
    /// Bytes sequence with the 0xfe prefix, but with length less than 254.
    BytesLenPrefix254LessThan254(u32),
    /// Padding for a bytes sequence that has at least one non-zero byte.
    NonZeroBytesPadding,
    /// This `serde` data format doesn't support several types in the Serde data model.
    UnsupportedSerdeType(DeSerdeType),
    /// Not enough elements, stores the already deserialized and expected count.
    NotEnoughElements(u32, u32),
    /// A wrong map key found while deserializing.
    InvalidMapKey(String, &'static str),
    /// A wrong type id found while deserializing.
    InvalidTypeId(u32, &'static [u32]),
    /// The deserialized type id and the one known from value aren't the same.
    TypeIdMismatch(u32, u32),
    /// No enum variant id found in the deserializer to continue deserialization.
    NoEnumVariantId,
    /// The deserialized size and the predicted one aren't the same.
    SizeMismatch(u32, u32),
}

impl fmt::Display for DeErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DeErrorKind::Msg(ref string) => {
                write!(f, "custom string: {}", string)
            },
            DeErrorKind::BytesLenPrefix254LessThan254(len) => {
                write!(f, "byte sequence has the 0xfe prefix with length less than 254 {}", len)
            },
            DeErrorKind::NonZeroBytesPadding => {
                write!(f, "byte sequence has a padding with a non-zero byte")
            },
            DeErrorKind::UnsupportedSerdeType(ref type_) => {
                write!(f, "{} type is not supported for deserialization", type_)
            },
            DeErrorKind::NotEnoughElements(deserialized_count, expected_count) => {
                write!(f, "not enough elements: have {}, need {}", deserialized_count, expected_count)
            },
            DeErrorKind::InvalidMapKey(ref found_key, expected_key) => {
                write!(f, "invalid map key {:?}, expected {:?}", found_key, expected_key)
            },
            DeErrorKind::InvalidTypeId(found_type_id, valid_type_ids) => {
                write!(f, "invalid type id {}, expected {:?}", found_type_id, valid_type_ids)
            },
            DeErrorKind::TypeIdMismatch(deserialized_type_id, static_type_id) => {
                write!(f, "type id mismatch: deserialized {}, but {} found from value",
                    deserialized_type_id, static_type_id)
            },
            DeErrorKind::NoEnumVariantId => {
                write!(f, "no enum variant id found in deserializer")
            },
            DeErrorKind::SizeMismatch(deserialized_size, static_size_hint) => {
                write!(f, "size mismatch: deserialized {}, predicted {}",
                    deserialized_size, static_size_hint)
            },
        }
    }
}

/// Serde deserialization data types that are not supported by `serde_mtproto`.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
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
