//! # Serde MTProto
//!
//! MTProto is a mobile-first protocol for access to a server API.
//! This crate provides means to serialize Rust types to its binary
//! representation and to deserialize from said representation.

#![deny(missing_docs)]

#![cfg_attr(all(not(stable_i128), feature = "i128"), feature(i128_type))]

#![cfg_attr(feature = "test-nightly-regressions", feature(nll))]

#![cfg_attr(feature = "cargo-clippy", deny(
    // Turn all warn-class lints to denies
    clippy,
))]

#![cfg_attr(feature = "cargo-clippy", warn(
    // Additional warns about numeric casts
    cast_possible_truncation,
    cast_possible_wrap,
    cast_precision_loss,
    cast_sign_loss,
    invalid_upcast_comparisons,

    // Other lints we consider useful to use as warns in this crate
    empty_enum,
    enum_glob_use,
    float_arithmetic,
    indexing_slicing,
    invalid_upcast_comparisons,
    mem_forget,
    mut_mut,
    print_stdout,
    result_unwrap_used,
    used_underscore_binding,
    wrong_pub_self_convention,
))]


extern crate byteorder;
#[macro_use]
extern crate error_chain;
#[cfg(feature = "extprim")]
extern crate extprim;
#[macro_use]
extern crate log;
extern crate num_traits;
#[cfg(feature = "quickcheck")]
extern crate quickcheck;
extern crate serde;
extern crate serde_bytes;
#[macro_use]
extern crate serde_derive;


mod utils;

pub mod de;
pub mod error;
pub mod helpers;
pub mod identifiable;
pub mod ser;
pub mod sized;
pub mod wrappers;


// Extern crate re-export for convenience
pub use serde_bytes::{ByteBuf, Bytes};

// Serde essential re-exports
pub use ser::{Serializer, to_bytes, to_writer, unsized_bytes_pad_to_bytes, unsized_bytes_pad_to_writer};
pub use de::{Deserializer, from_bytes, from_bytes_reuse, from_reader, from_reader_reuse};

// Error types and typedefs
pub use error::{Error, ErrorKind, Result, ResultExt};

// Other items generally useful for MTProto [de]serialization
pub use helpers::{UnsizedByteBuf, UnsizedByteBufSeed, UnsizedBytes, size_hint_from_unsized_byte_seq_len};
pub use identifiable::Identifiable;
pub use sized::{MtProtoSized, size_hint_from_byte_seq_len};
pub use wrappers::{Boxed, BoxedWithSize, WithSize};
