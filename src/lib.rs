//! # Serde MTProto
//!
//! MTProto is a mobile-first protocol for access to a server API.
//! This crate provides means to serialize Rust types to its binary
//! representation and to deserialize from said representation.

// For `error_chain!` macro used in `error` module
#![recursion_limit = "66"]

#![cfg_attr(all(not(stable_i128), feature = "i128"), feature(i128_type))]
#![cfg_attr(feature = "test-nightly-regressions", feature(nll))]


// ========== RUSTC LINTS ========== //

#![cfg_attr(feature = "aggressive-rustc-lints", deny(
    // Deny some warn-level lints
    const_err,
    deprecated,
    improper_ctypes,
    overflowing_literals,
    patterns_in_fns_without_body,
    private_no_mangle_fns,
    private_no_mangle_statics,
    renamed_and_removed_lints,
    unconditional_recursion,
    unions_with_drop_fields,
    unknown_lints,
    while_true,

    // Deny some allow-level lints
    missing_debug_implementations,
    missing_docs,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    unused_results,
))]

#![cfg_attr(all(feature = "aggressive-rustc-lints", lints_1_19), deny(
    // Deny some warn-level lints available from Rust 1.19
    illegal_floating_point_literal_pattern,

    // Deny some allow-level lints available from Rust 1.19
    anonymous_parameters,
))]

#![cfg_attr(all(feature = "aggressive-rustc-lints", lints_1_24), deny(
    // Deny some warn-level lints available from Rust 1.24
    safe_packed_borrows,
    tyvar_behind_raw_pointer,
))]

#![cfg_attr(all(feature = "aggressive-rustc-lints", lints_1_26, not(lints_1_27)), deny(
    // Deny some warn-level lints available from Rust 1.26 that are renamed in
    // 1.27 (relevant PR: https://github.com/rust-lang/rust/pull/50879).
    unstable_name_collision,
))]

#![cfg_attr(all(feature = "aggressive-rustc-lints", lints_1_27), deny(
    // Deny some warn-level lints available from previous Rust versions that are
    // renamed in 1.27
    // (relevant PR: https://github.com/rust-lang/rust/pull/50879).
    unstable_name_collisions,
))]


// ========== CLIPPY LINTS ========== //

#![cfg_attr(feature = "cargo-clippy", warn(
    // Warn about every lint in these categories (not deny because some them may have false
    // positives according to `clippy` README)
    clippy_complexity,
    clippy_correctness,
    clippy_perf,
    clippy_style,

    // Restrict our code to ease reviewing and auditing in some cases
    clone_on_ref_ptr,
    decimal_literal_representation,
    float_arithmetic,
    indexing_slicing,
    mem_forget,
    print_stdout,
    result_unwrap_used,
    shadow_unrelated,
    wrong_pub_self_convention,

    // Additional pedantic warns about numeric casts
    cast_possible_truncation,
    cast_possible_wrap,
    cast_precision_loss,
    cast_sign_loss,
    invalid_upcast_comparisons,

    // Other pedantic lints we consider useful to use as warns in this crate
    doc_markdown,
    empty_enum,
    enum_glob_use,
    items_after_statements,
    match_same_arms,
    maybe_infinite_iter,
    mut_mut,
    needless_continue,
    pub_enum_variant_names,
    similar_names,
    string_add_assign,
    unseparated_literal_suffix,
    used_underscore_binding,
))]

#![cfg_attr(feature = "cargo-clippy", deny(
    // Turn all warn-level lints that have no false positives (according to `clippy` README) to
    // denies (because it should be safe to do so)
    clippy,
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
        from_reader,
        from_reader_reuse,
    };

    // Error types and typedefs
    pub use error::{Error, ErrorKind, Result, ResultExt};

    // Other items generally useful for MTProto [de]serialization
    pub use helpers::{
        UnsizedByteBuf,
        UnsizedByteBufSeed,
        UnsizedBytes,
        size_hint_from_unsized_byte_seq_len,
    };
    pub use identifiable::Identifiable;
    pub use sized::{MtProtoSized, size_hint_from_byte_seq_len};
    pub use wrappers::{Boxed, WithId, WithSize};
}
