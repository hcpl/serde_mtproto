//! # Serde MTProto
//!
//! MTProto is a mobile-first protocol for access to a server API.
//! This crate provides means to serialize Rust types to its binary
//! representation and to deserialize from said representation.


// For `error_chain!` macro used in `error` module
#![recursion_limit = "66"]

#![cfg_attr(feature = "test-nightly-regressions", feature(nll))]


// ========== RUSTC LINTS ========== //

#![warn(
    // Warn some allow-level lints
    anonymous_parameters,
    missing_debug_implementations,
    missing_docs,
    rust_2018_idioms,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unstable_name_collisions,
    unused_import_braces,
    unused_results,
)]


// ========== CLIPPY LINTS ========== //

#![cfg_attr(feature = "cargo-clippy", warn(
    // Restrict our code to ease reviewing and auditing in some cases
    clippy::clone_on_ref_ptr,
    clippy::decimal_literal_representation,
    clippy::float_arithmetic,
    clippy::indexing_slicing,
    clippy::mem_forget,
    clippy::print_stdout,
    clippy::result_unwrap_used,
    clippy::shadow_unrelated,
    clippy::wrong_pub_self_convention,

    // Additional pedantic warns about numeric casts
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::invalid_upcast_comparisons,

    // Other pedantic lints we consider useful to use as warns in this crate
    clippy::doc_markdown,
    clippy::empty_enum,
    clippy::enum_glob_use,
    clippy::items_after_statements,
    clippy::match_same_arms,
    clippy::maybe_infinite_iter,
    clippy::mut_mut,
    clippy::needless_continue,
    clippy::pub_enum_variant_names,
    clippy::similar_names,
    clippy::string_add_assign,
    clippy::unseparated_literal_suffix,
    clippy::used_underscore_binding,
))]


// Workaround for <https://github.com/rust-lang/rust/issues/55779>
#[allow(unused_extern_crates)]
extern crate serde;


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

macro_rules! doc_inline {
    ($($i:item)*) => ($(#[doc(inline)] $i)*)
}

doc_inline! {
    // Serde essential re-exports
    pub use crate::ser::{
        Serializer,
        to_bytes,
        to_writer,
        unsized_bytes_pad_to_bytes,
        unsized_bytes_pad_to_writer,
    };
    pub use crate::de::{
        Deserializer,
        from_bytes,
        from_bytes_reuse,
        from_bytes_seed,
        from_reader,
        from_reader_reuse,
        from_reader_seed,
    };

    // Error types and typedefs
    pub use crate::error::{Error, ErrorKind, Result, ResultExt};

    // Other items generally useful for MTProto [de]serialization
    pub use crate::helpers::{UnsizedByteBuf, UnsizedByteBufSeed};
    pub use crate::identifiable::Identifiable;
    pub use crate::sized::{MtProtoSized, size_hint_from_byte_seq_len};
    pub use crate::wrappers::{Boxed, WithId, WithSize};
}
