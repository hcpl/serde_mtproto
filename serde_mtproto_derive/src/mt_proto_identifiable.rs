use quote;
use syn::{Attribute, Body, DeriveInput, Ident, Lit, MetaItem, StrStyle};


pub fn impl_mt_proto_identifiable(ast: &DeriveInput) -> quote::Tokens {
    let item_name = &ast.ident;
    let (item_impl_generics, item_ty_generics, item_where_clause) = ast.generics.split_for_impl();

    let dummy_const = Ident::new(format!("_IMPL_MT_PROTO_IDENTIFIABLE_FOR_{}", item_name));

    let get_id_body = match ast.body {
        Body::Struct(_) => {
            let id = get_id_from_attrs(&ast.attrs);

            quote! {
                #id
            }
        }

        Body::Enum(ref variants) => {
            let mut variants_quoted = quote::Tokens::new();

            for variant in variants {
                let variant_name = &variant.ident;
                let id = get_id_from_attrs(&variant.attrs);

                variants_quoted.append(quote! {
                    #item_name::#variant_name { .. } => #id,
                });
            }

            quote! {
                match *self {
                    #variants_quoted
                }
            }
        }
    };

    let get_enum_variant_id_body = match ast.body {
        Body::Struct(_) => {
            quote! {
                None
            }
        }

        Body::Enum(ref variants) => {
            let mut variants_quoted = quote::Tokens::new();

            for variant in variants {
                let variant_name = &variant.ident;

                variants_quoted.append(quote! {
                    #item_name::#variant_name { .. } => stringify!(#variant_name),
                });
            }

            quote! {
                let variant_id = match *self {
                    #variants_quoted
                };

                Some(variant_id)
            }
        }
    };

    quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate serde_mtproto as _serde_mtproto;

            impl #item_impl_generics _serde_mtproto::Identifiable for #item_name #item_ty_generics
                #item_where_clause
            {
                fn get_id(&self) -> i32 {
                    #get_id_body
                }

                fn get_enum_variant_id(&self) -> Option<&'static str> {
                    #get_enum_variant_id_body
                }
            }
        };
    }
}

fn get_id_from_attrs(attrs: &[Attribute]) -> i32 {
    for attr in attrs {
        match attr.value {
            MetaItem::NameValue(ref name, ref value) => {
                if name.as_ref() == "id" {
                    if let Lit::Str(ref value, StrStyle::Cooked) = *value {
                        // Found an identifier
                        let value = u32::from_str_radix(&value[2..], 16).unwrap();

                        return value as i32;
                    }
                }
            }

            _ => {
                // Do nothing
            }
        }
    }

    panic!("#[derive(MtProtoIdentifiable)] requires an #[id = \"0x...\"] attribute:\n    \
            - on top of struct for structs;\n    \
            - or on top of each enum variant for enums");
}
