use proc_macro2;
use quote::ToTokens;
use syn;

use ast;
use ext::IteratorResultExt;


pub(crate) fn impl_mt_proto_identifiable(container: ast::Container) -> proc_macro2::TokenStream {
    match impl_mt_proto_identifiable_or_error(container) {
        Ok(tokens) => tokens,
        Err(e) => e.iter().map(syn::Error::to_compile_error).collect(),
    }
}

fn impl_mt_proto_identifiable_or_error(
    container: ast::Container,
) -> Result<proc_macro2::TokenStream, Vec<syn::Error>> {
    let (item_impl_generics, item_ty_generics, item_where_clause) =
        container.generics.split_for_impl();

    let item_name = &container.ident;

    let dummy_const = ident!("_IMPL_MT_PROTO_IDENTIFIABLE_FOR_{}", item_name);

    let all_type_ids_value = match container.data {
        ast::Data::Struct(_) => {
            let id = get_id_from_attrs(&container.attrs, (&container).into_token_stream())
                .map_err(|e| vec![e])?;

            quote!(&[#id])
        },
        ast::Data::Enum(ref data_enum) => {
            let ids = data_enum.variants
                .iter()
                .map(|v| get_id_from_attrs(&v.attrs, v.into_token_stream()))
                .collect_results()?;

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
            let id = get_asserted_id_from_attrs(&container.attrs, (&container).into_token_stream())
                .map_err(|e| vec![e])?;

            quote!(#id)
        },
        ast::Data::Enum(ref data_enum) => {
            let variants = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let id = get_asserted_id_from_attrs(&variant.attrs, variant.into_token_stream())?;

                Ok(quote! {
                    #item_name::#variant_name { .. } => #id,
                })
            }).collect_results()?;

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

    Ok(quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate serde_mtproto as _serde_mtproto;

            impl #item_impl_generics _serde_mtproto::Identifiable for #item_name #item_ty_generics
                #item_where_clause
            {
                fn all_type_ids() -> &'static [u32] {
                    #all_type_ids_value
                }

                fn all_enum_variant_names() -> Option<&'static [&'static str]> {
                    #all_enum_variant_names_value
                }

                fn type_id(&self) -> u32 {
                    #type_id_body
                }

                fn enum_variant_id(&self) -> Option<&'static str> {
                    #enum_variant_id_body
                }
            }
        };
    })
}


fn get_asserted_id_from_attrs(
    attrs: &[syn::Attribute],
    input_tokens: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    let id = get_id_from_attrs(attrs, input_tokens)?;
    let check_expr = quote!(Self::all_type_ids().contains(&#id));

    control_flow_chain! {
        for attr in attrs;
        if let syn::AttrStyle::Outer = attr.style;
        if let Ok(syn::Meta::List(list)) = attr.parse_meta();
        if list.ident == "mtproto_identifiable";
        for nested_meta in list.nested;
        if let syn::NestedMeta::Meta(syn::Meta::List(nested_list)) = nested_meta;
        if nested_list.ident == "check_type_id";
        then {
            if nested_list.nested.len() != 1 {
                return Err(syn::Error::new_spanned(
                    nested_list,
                    "`check_type_id(...)` must have exactly 1 parameter",
                ));
            }

            if let syn::NestedMeta::Meta(syn::Meta::Word(ref ident)) = nested_list.nested[0] {
                let res = match ident.to_string().as_ref() {
                    "never" => quote!(#id),
                    "debug_only" => quote!({ debug_assert!(#check_expr); #id }),
                    "always" => quote!({ assert!(#check_expr); #id }),
                    _ => continue,
                };

                return Ok(res);
            }
        }
    }

    Ok(quote!({ assert!(#check_expr); #id }))
}

fn get_id_from_attrs(
    attrs: &[syn::Attribute],
    input_tokens: proc_macro2::TokenStream,
) -> syn::Result<u32> {
    control_flow_chain! {
        for attr in attrs;
        if let syn::AttrStyle::Outer = attr.style;
        if let Ok(syn::Meta::List(list)) = attr.parse_meta();
        if list.ident == "mtproto_identifiable";
        for nested_meta in list.nested;
        if let syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) = nested_meta;
        if name_value.ident == "id";
        then {
            if let syn::Lit::Str(lit_str) = name_value.lit {
                // Found an identifier
                let str_value = lit_str.value();

                if str_value.len() >= 2 {
                    match str_value.split_at(2) {
                        ("0x", hex) => return Ok(u32::from_str_radix(hex, 16).unwrap()),
                        ("0b", bin) => return Ok(u32::from_str_radix(bin, 2).unwrap()),
                        ("0o", oct) => return Ok(u32::from_str_radix(oct, 8).unwrap()),
                        _ => (),
                    }
                }

                return Ok(u32::from_str_radix(&str_value, 10).unwrap());
            } else {
                return Err(syn::Error::new_spanned(
                    name_value.lit,
                    "expected mtproto id attribute to be a string: `id = \"...\"`",
                ));
            }
        }
    }

    const ERROR_MESSAGE: &str = "\
        #[derive(MtProtoIdentifiable)] requires an #[mtproto_identifiable(id = \"...\")] attribute\n    \
        where id can can be either:\n    \
        - hexadecimal with 0x prefix,\n    \
        - binary with 0b,\n    \
        - octal with 0o\n    \
        - or decimal with no prefix.";

    Err(syn::Error::new_spanned(input_tokens, ERROR_MESSAGE))
}
