use proc_macro2;
use syn;

use ast;


pub(crate) fn impl_mt_proto_identifiable(container: ast::Container) -> proc_macro2::TokenStream {
    let (item_impl_generics, item_ty_generics, item_where_clause) =
        container.generics.split_for_impl();

    let item_name = &container.ident;

    let dummy_const = ident!("_IMPL_MT_PROTO_IDENTIFIABLE_FOR_{}", item_name);
    let all_type_ids_const = ident!("_ALL_TYPE_IDS_OF_{}", item_name);
    let all_enum_variant_names_const = ident!("_ALL_ENUM_VARIANT_NAMES_OF_{}", item_name);

    let all_type_ids_value = match container.data {
        ast::Data::Struct(_) => {
            let id = get_id_from_attrs(&container.attrs);

            quote!(&[#id])
        },
        ast::Data::Enum(ref data_enum) => {
            let ids = data_enum.variants
                .iter()
                .map(|v| get_id_from_attrs(&v.attrs));

            quote!(&[#(#ids),*])
        },
    };

    let all_enum_variant_names_value = match container.data {
        ast::Data::Struct(_) => {
            quote!(None)
        },
        ast::Data::Enum(ref data_enum) => {
            let names = data_enum.variants
                .iter()
                .map(|v| proc_macro2::Literal::string(&v.ident.to_string()));

            quote!(Some(&[#(#names),*]))
        },
    };

    let type_id_body = match container.data {
        ast::Data::Struct(_) => {
            let id = get_asserted_id_from_attrs(&container.attrs);

            quote!(#id)
        },
        ast::Data::Enum(ref data_enum) => {
            let variants = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let id = get_asserted_id_from_attrs(&variant.attrs);

                quote! {
                    #item_name::#variant_name { .. } => #id,
                }
            });

            quote! {
                match *self {
                    #(#variants)*
                }
            }
        },
    };

    let enum_variant_id_body = match container.data {
        ast::Data::Struct(_) => {
            quote! { None }
        },
        ast::Data::Enum(ref data_enum) => {
            let variants = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let variant_name_string = proc_macro2::Literal::string(&variant_name.to_string());

                quote! {
                    #item_name::#variant_name { .. } => #variant_name_string,
                }
            });

            quote! {
                let variant_id = match *self {
                    #(#variants)*
                };

                Some(variant_id)
            }
        },
    };

    // TODO: Use rvalue static promotion syntax after bumping minimum supported Rust version to 1.21
    quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate serde_mtproto as _serde_mtproto;

            const #all_type_ids_const: &'static [u32] = #all_type_ids_value;
            const #all_enum_variant_names_const: Option<&'static [&'static str]> = #all_enum_variant_names_value;

            impl #item_impl_generics _serde_mtproto::Identifiable for #item_name #item_ty_generics
                #item_where_clause
            {
                fn all_type_ids() -> &'static [u32] {
                    #all_type_ids_const
                }

                fn all_enum_variant_names() -> Option<&'static [&'static str]> {
                    #all_enum_variant_names_const
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


fn get_asserted_id_from_attrs(attrs: &[syn::Attribute]) -> proc_macro2::TokenStream {
    let id = get_id_from_attrs(attrs);
    let check_expr = quote!(Self::all_type_ids().contains(&#id));

    for attr in attrs {
        if let syn::AttrStyle::Inner(..) = attr.style {
            continue;
        }

        if let Some(syn::Meta::List(list)) = attr.interpret_meta() {
            if list.ident == "mtproto_identifiable" {
                for nested_meta in list.nested {
                    if let syn::NestedMeta::Meta(syn::Meta::List(nested_list)) = nested_meta {
                        if nested_list.ident == "check_type_id" {
                            if nested_list.nested.len() != 1 {
                                panic!("check_type_id must have exactly 1 argument");
                            }

                            if let syn::NestedMeta::Meta(syn::Meta::Word(ref ident)) =
                                nested_list.nested[0]
                            {
                                let res = match ident.to_string().as_ref() {
                                    "never" => quote!(#id),
                                    "debug_only" => quote!({ debug_assert!(#check_expr); #id }),
                                    "always" => quote!({ assert!(#check_expr); #id }),
                                    _ => continue,
                                };

                                return res;
                            }
                        }
                    }
                }
            }
        }
    }

    quote!({ assert!(#check_expr); #id })
}

fn get_id_from_attrs(attrs: &[syn::Attribute]) -> u32 {
    for attr in attrs {
        if let syn::AttrStyle::Inner(..) = attr.style {
            continue;
        }

        if let Some(syn::Meta::List(list)) = attr.interpret_meta() {
            if list.ident == "mtproto_identifiable" {
                for nested_meta in list.nested {
                    if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) = nested_meta {
                        if name_value.ident == "id" {
                            if let syn::Lit::Str(lit_str) = name_value.lit {
                                // Found an identifier
                                let str_value = lit_str.value();

                                if str_value.len() >= 2 {
                                    match str_value.split_at(2) {
                                        ("0x", hex) => return u32::from_str_radix(hex, 16).unwrap(),
                                        ("0b", bin) => return u32::from_str_radix(bin, 2).unwrap(),
                                        ("0o", oct) => return u32::from_str_radix(oct, 8).unwrap(),
                                        _ => (),
                                    }
                                }

                                return u32::from_str_radix(&str_value, 10).unwrap();
                            } else {
                                panic!("`id` attribute must have a `str` value.");
                            }
                        }
                    }
                }
            }
        }
    }

    panic!("#[derive(MtProtoIdentifiable)] requires an #[mtproto_identifiable(id = \"...\")] attribute:\n    \
            - on top of struct for structs;\n    \
            - or on top of each enum variant for enums.\n\
            id can can be either:\n    \
            - hexadecimal with 0x prefix,\n    \
            - binary with 0b,\n    \
            - octal with 0o\n    \
            - or decimal with no prefix.");
}
