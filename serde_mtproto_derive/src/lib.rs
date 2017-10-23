//! This crate provides Serde MTProto's two derive macros.
//!
//! ```
//! # #[macro_use] extern crate serde_mtproto_derive;
//! #[derive(MtProtoIdentifiable, MtProtoSized)]
//! # #[id = "0x00000000"]
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
//! #[id = "0xbeefdead"]
//! struct Message {
//!     message_id: u32,
//!     user_id: u32,
//!     text: String,
//!     attachment: Attachment,
//! }
//!
//! #[derive(MtProtoIdentifiable, MtProtoSized)]
//! enum Attachment {
//!     #[id = "0xdef19e00"]
//!     Nothing,
//!     #[id = "0xbadf00d0"]
//!     Link {
//!         url: String,
//!     },
//!     #[id = "0xdeafbeef"]
//!     Repost {
//!         message_id: u32,
//!     },
//! }
//!
//! # fn main() {}
//! ```

extern crate proc_macro;
#[macro_use]
extern crate quote;
extern crate syn;


mod mt_proto_identifiable;
mod mt_proto_sized;


use proc_macro::TokenStream;

use mt_proto_identifiable::impl_mt_proto_identifiable;
use mt_proto_sized::impl_mt_proto_sized;


#[proc_macro_derive(MtProtoIdentifiable, attributes(id))]
pub fn mt_proto_identifiable(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_mt_proto_identifiable(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}

#[proc_macro_derive(MtProtoSized, attributes(mtproto_sized))]
pub fn mt_proto_sized(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let mut ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_mt_proto_sized(&mut ast);

    // Return the generated impl
    gen.parse().unwrap()
}
