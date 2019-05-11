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

// For `quote!` and `control_flow_chain!` macros
#![recursion_limit = "79"]

// This lint is not compatible with defensive programming, let's disable it
#![cfg_attr(feature = "cargo-clippy", allow(clippy::unneeded_field_pattern))]

extern crate proc_macro;


#[macro_use]
mod macros;

mod ast;
mod ext;
mod identifiable;
mod sized;


use proc_macro::TokenStream;


#[proc_macro_derive(MtProtoIdentifiable, attributes(mtproto_identifiable))]
pub fn mt_proto_identifiable(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let tokens = match ast::Container::from_derive_input(ast, "mtproto::Identifiable") {
        Ok(container) => crate::identifiable::impl_derive(container),
        Err(e) => e.to_compile_error(),
    };

    tokens.into()
}

#[proc_macro_derive(MtProtoSized, attributes(mtproto_sized))]
pub fn mt_proto_sized(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let tokens = match ast::Container::from_derive_input(ast, "mtproto::MtProtoSized") {
        Ok(container) => crate::sized::impl_derive(container),
        Err(e) => e.to_compile_error(),
    };

    tokens.into()
}
