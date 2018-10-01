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
#![recursion_limit = "87"]

extern crate proc_macro;

extern crate proc_macro2;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;


mod mt_proto_identifiable;
mod mt_proto_sized;


use proc_macro::TokenStream;

use mt_proto_identifiable::impl_mt_proto_identifiable;
use mt_proto_sized::impl_mt_proto_sized;


#[proc_macro_derive(MtProtoIdentifiable, attributes(mtproto_identifiable))]
pub fn mt_proto_identifiable(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as syn::DeriveInput);
    let res = impl_mt_proto_identifiable(&ast);
    res.into()
}

#[proc_macro_derive(MtProtoSized, attributes(mtproto_sized))]
pub fn mt_proto_sized(input: TokenStream) -> TokenStream {
    let mut ast = parse_macro_input!(input as syn::DeriveInput);
    let res = impl_mt_proto_sized(&mut ast);
    res.into()
}
