use proc_macro2::Span;
use quote;
use syn::{Attribute, AttrStyle, Data, DeriveInput, Ident, Lit, Meta, NestedMeta};


pub fn impl_mt_proto_identifiable(ast: &DeriveInput) -> quote::Tokens {
    let (item_impl_generics, item_ty_generics, item_where_clause) = ast.generics.split_for_impl();

    let item_name = &ast.ident;
    let dummy_const =
        Ident::new(&format!("_IMPL_MT_PROTO_IDENTIFIABLE_FOR_{}", item_name), Span::call_site());
    let all_type_ids_const =
        Ident::new(&format!("_ALL_TYPE_IDS_OF_{}", item_name), Span::call_site());

    let all_type_ids_value = match ast.data {
        Data::Struct(_) => {
            let id = get_id_from_attrs(&ast.attrs);

            quote! { &[#id] }
        },
        Data::Enum(ref data_enum) => {
            let ids = data_enum.variants.iter()
                .map(|v| get_id_from_attrs(&v.attrs))
                .collect::<Vec<_>>();

            quote! { &[#(#ids),*] }
        },
        Data::Union(_) => panic!("Cannot derive `mtproto::Identifiable` for unions."),
    };

    let type_id_body = match ast.data {
        Data::Struct(_) => {
            let id = get_asserted_id_from_attrs(&ast.attrs);

            quote! { #id }
        },
        Data::Enum(ref data_enum) => {
            let mut variants_quoted = quote::Tokens::new();

            for variant in &data_enum.variants {
                let variant_name = &variant.ident;
                let id = get_asserted_id_from_attrs(&variant.attrs);

                variants_quoted.append_all(&[quote! {
                    #item_name::#variant_name { .. } => #id,
                }]);
            }

            quote! {
                match *self {
                    #variants_quoted
                }
            }
        },
        Data::Union(_) => panic!("Cannot derive `mtproto::Identifiable` for unions."),
    };

    let enum_variant_id_body = match ast.data {
        Data::Struct(_) => {
            quote! { None }
        },
        Data::Enum(ref data_enum) => {
            let mut variants_quoted = quote::Tokens::new();

            for variant in &data_enum.variants {
                let variant_name = &variant.ident;

                variants_quoted.append_all(&[quote! {
                    #item_name::#variant_name { .. } => stringify!(#variant_name),
                }]);
            }

            quote! {
                let variant_id = match *self {
                    #variants_quoted
                };

                Some(variant_id)
            }
        },
        Data::Union(_) => panic!("Cannot derive `mtproto::Identifiable` for unions."),
    };

    // TODO: Use rvalue static promotion syntax after bumping minimum supported Rust version to 1.21
    quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate serde_mtproto as _serde_mtproto;

            const #all_type_ids_const: &'static [u32] = #all_type_ids_value;

            impl #item_impl_generics _serde_mtproto::Identifiable for #item_name #item_ty_generics
                #item_where_clause
            {
                fn all_type_ids() -> &'static [u32] {
                    #all_type_ids_const
                }

                fn type_id(&self) -> u32 {
                    #type_id_body
                }

                fn enum_variant_id(&self) -> Option<&'static str> {
                    #enum_variant_id_body
                }
            }
        };
    }
}


fn get_asserted_id_from_attrs(attrs: &[Attribute]) -> quote::Tokens {
    let id = get_id_from_attrs(attrs);
    let check_expr = quote! {
        Self::all_type_ids().contains(&#id)
    };

    for attr in attrs {
        if let Attribute {
            style: AttrStyle::Outer,
            is_sugared_doc: false,
            ..
        } = *attr {
            if let Some(Meta::List(ref meta_list)) = attr.interpret_meta() {
                if meta_list.ident == "check_type_id" && meta_list.nested.len() == 1 {
                    if let NestedMeta::Meta(Meta::Word(ref ident)) = meta_list.nested[0] {
                        let res = match ident.as_ref() {
                            "never" => quote! { #id },
                            "debug_only" => quote! {
                                {
                                    debug_assert!(#check_expr);
                                    #id
                                }
                            },
                            "always" => quote! {
                                {
                                    assert!(#check_expr);
                                    #id
                                }
                            },
                            _ => continue,
                        };

                        return res;
                    }
                }
            }
        }
    }

    quote! {
        {
            assert!(#check_expr);
            #id
        }
    }
}

fn get_id_from_attrs(attrs: &[Attribute]) -> u32 {
    for attr in attrs {
        if let Attribute {
            style: AttrStyle::Outer,
            is_sugared_doc: false,
            ..
        } = *attr {
            if let Some(Meta::NameValue(ref meta_name_value)) = attr.interpret_meta() {
                if meta_name_value.ident == "id" {
                    if let Lit::Str(ref lit_str) = meta_name_value.lit {
                        // Found an identifier
                        let str_value = lit_str.value();

                        let value = if str_value.starts_with("0x") {
                            u32::from_str_radix(&str_value[2..], 16).unwrap()
                        } else if str_value.starts_with("0b") {
                            u32::from_str_radix(&str_value[2..], 2).unwrap()
                        } else if str_value.starts_with("0o") {
                            u32::from_str_radix(&str_value[2..], 8).unwrap()
                        } else {
                            u32::from_str_radix(&str_value, 10).unwrap()
                        };

                        return value;
                    } else {
                        panic!("`id` attribute must have a `str` value.");
                    }
                }
            }
        }
    }

    panic!("#[derive(MtProtoIdentifiable)] requires an #[id = \"...\"] attribute:\n    \
            - on top of struct for structs;\n    \
            - or on top of each enum variant for enums.\n\
            id can can be either:\n    \
            - hexadecimal with 0x prefix,\n    \
            - binary with 0b,\n    \
            - octal with 0o\n    \
            - or decimal with no prefix.");
}
