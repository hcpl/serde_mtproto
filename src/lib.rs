//! # Serde MTProto
//!
//! MTProto is a mobile-first protocol for access to a server API.
//! This crate provides means to serialize Rust types to its binary
//! representation and to deserialize from said representation.


// For `error_chain!` macro used in `error` module
#![recursion_limit = "66"]

// See <https://github.com/rust-lang/rust/issues/50907> for details.
#![cfg_attr(feature = "nightly", feature(exhaustive_integer_patterns))]
#![cfg_attr(feature = "test-nightly-regressions", feature(nll))]


// ========== RUSTC LINTS ========== //

#![deny(
    // Deny some warn-level lints
    const_err,
    deprecated,
    illegal_floating_point_literal_pattern,
    improper_ctypes,
    late_bound_lifetime_arguments,
    non_camel_case_types,
    non_shorthand_field_patterns,
    non_snake_case,
    non_upper_case_globals,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    renamed_and_removed_lints,
    safe_packed_borrows,
    tyvar_behind_raw_pointer,
    unconditional_recursion,
    unions_with_drop_fields,
    unknown_lints,
    unreachable_code,
    unreachable_patterns,
    unused_allocation,
    unused_features,
    unused_unsafe,
    while_true,

    // Deny some allow-level lints
    anonymous_parameters,
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unstable_name_collisions,
    unused_import_braces,
    unused_results,
)]


// ========== CLIPPY LINTS ========== //

#![cfg_attr(feature = "cargo-clippy", warn(
    // Restrict our code to ease reviewing and auditing in some cases
    clippy::decimal_literal_representation,
    clippy::float_arithmetic,

    // Additional pedantic warns about numeric casts
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

#![cfg_attr(feature = "cargo-clippy", deny(
    // Turn all warn-level lints that have no false positives (according to `clippy` README) to
    // denies (because it should be safe to do so)
    clippy::all,

    // Restrict our code to ease reviewing and auditing in some cases
    clippy::clone_on_ref_ptr,
    clippy::indexing_slicing,
    clippy::mem_forget,
    clippy::print_stdout,
    clippy::result_unwrap_used,
    clippy::shadow_unrelated,
    clippy::wrong_pub_self_convention,

    // Additional pedantic denies about numeric casts
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
))]


extern crate byteorder;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate num_traits;
#[cfg(feature = "quickcheck")]
#[cfg_attr(test, macro_use)]
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

macro_rules! doc_inline {
    ($($i:item)*) => ($(#[doc(inline)] $i)*)
}

doc_inline! {
    // Serde essential re-exports
    pub use ser::{
        Serializer,
        to_bytes,
        to_writer,
        unsized_bytes_pad_to_bytes,
        unsized_bytes_pad_to_writer,
    };
    pub use de::{
        Deserializer,
        from_bytes,
        from_bytes_reuse,
        from_bytes_seed,
        from_reader,
        from_reader_reuse,
        from_reader_seed,
    };

    // Error types and typedefs
    pub use error::{Error, ErrorKind, Result, ResultExt};

    // Other items generally useful for MTProto [de]serialization
    pub use helpers::{UnsizedByteBuf, UnsizedByteBufSeed};
    pub use identifiable::Identifiable;
    pub use sized::{MtProtoSized, size_hint_from_byte_seq_len};
    pub use wrappers::{Boxed, WithId, WithSize};
}
