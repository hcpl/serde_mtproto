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

#[proc_macro_derive(MtProtoSized)]
pub fn mt_proto_sized(input: TokenStream) -> TokenStream {
    // Construct a string representation of the type definition
    let s = input.to_string();

    // Parse the string representation
    let ast = syn::parse_derive_input(&s).unwrap();

    // Build the impl
    let gen = impl_mt_proto_sized(&ast);

    // Return the generated impl
    gen.parse().unwrap()
}
