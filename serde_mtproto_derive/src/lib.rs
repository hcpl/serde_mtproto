//! This crate provides Serde MTProto's two derive macros.
//!
//! ```
//! # #[macro_use] extern crate serde_mtproto_derive;
//! #[derive(MtProtoIdentifiable, MtProtoSized)]
//! # #[mtproto_identifiable(id = "0x00000000")]
//! # struct Stub;
//! # fn main() {}
//! ```
//!
//! # Examples
//!
//! ```
//! extern crate serde_mtproto;
//! #[macro_use]
//! extern crate serde_mtproto_derive;
//!
//! #[derive(MtProtoIdentifiable, MtProtoSized)]
//! #[mtproto_identifiable(id = "0xbeefdead")]
//! struct Message {
//!     message_id: u32,
//!     user_id: u32,
//!     text: String,
//!     attachment: Attachment,
//! }
//!
//! #[derive(MtProtoIdentifiable, MtProtoSized)]
//! enum Attachment {
//!     #[mtproto_identifiable(id = "0xdef19e00")]
//!     Nothing,
//!     #[mtproto_identifiable(id = "0xbadf00d0")]
//!     Link {
//!         url: String,
//!     },
//!     #[mtproto_identifiable(id = "0xdeafbeef")]
//!     Repost {
//!         message_id: u32,
//!     },
//! }
//!
//! # fn main() {}
//! ```

// For `quote!` used at the end of `impl_mt_proto_identifiable`
#![recursion_limit = "66"]

#![cfg_attr(feature = "cargo-clippy", deny(clippy::all))]
// This lint is not compatible with defensive programming, let's disable it
#![cfg_attr(feature = "cargo-clippy", allow(clippy::unneeded_field_pattern))]

extern crate proc_macro;

extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;


#[macro_use]
mod macros;

mod ast;
mod ext;
mod identifiable;
mod sized;


use proc_macro::TokenStream;

use identifiable::impl_mt_proto_identifiable;
use sized::impl_mt_proto_sized;


#[proc_macro_derive(MtProtoIdentifiable, attributes(mtproto_identifiable))]
pub fn mt_proto_identifiable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let tokens = match ast::Container::from_derive_input(ast, "mtproto::Identifiable") {
        Ok(container) => impl_mt_proto_identifiable(container),
        Err(e) => e.to_compile_error(),
    };

    tokens.into()
}

#[proc_macro_derive(MtProtoSized, attributes(mtproto_sized))]
pub fn mt_proto_sized(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let tokens = match ast::Container::from_derive_input(ast, "mtproto::MtProtoSized") {
        Ok(container) => impl_mt_proto_sized(container),
        Err(e) => e.to_compile_error(),
    };

    tokens.into()
}
