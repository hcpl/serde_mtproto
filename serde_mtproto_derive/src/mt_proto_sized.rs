use quote;
use syn;


pub fn impl_mt_proto_sized(ast: &mut syn::DeriveInput) -> quote::Tokens {
    add_mt_proto_sized_trait_bound_if_missing(ast);
    let (item_impl_generics, item_ty_generics, item_where_clause) = ast.generics.split_for_impl();

    let item_name = &ast.ident;
    let dummy_const = syn::Ident::new(format!("_IMPL_MT_PROTO_SIZED_FOR_{}", item_name));

    let size_hint_body = match ast.body {
        syn::Body::Struct(ref data) => {
            let mut fields_quoted = quote! { 0 };

            match *data {
                syn::VariantData::Struct(ref fields) => {
                    for field in fields {
                        if is_skippable_field(field) {
                            continue;
                        }

                        let field_name = &field.ident;

                        fields_quoted.append(quote! {
                            + _serde_mtproto::MtProtoSized::size_hint(&self.#field_name)?
                        });
                    }
                }

                syn::VariantData::Tuple(ref fields) => {
                    for (i, field) in fields.iter().enumerate() {
                        if is_skippable_field(field) {
                            continue;
                        }

                        // Integers are rendered with type suffixes. We don't want this.
                        let i = quote::Ident::new(i.to_string());

                        fields_quoted.append(quote! {
                            + _serde_mtproto::MtProtoSized::size_hint(&self.#i)?
                        });
                    }
                }

                syn::VariantData::Unit => {}
            }

            quote! {
                Ok(#fields_quoted)
            }
        }

        syn::Body::Enum(ref variants) => {
            let mut variants_quoted = quote::Tokens::new();

            for variant in variants {
                let variant_name = &variant.ident;

                let pattern_match_quoted;
                let mut fields_quoted = quote! { 0 };

                match variant.data {
                    syn::VariantData::Struct(ref fields) => {
                        let mut pattern_matches = Vec::new();

                        for field in fields {
                            if is_skippable_field(field) {
                                continue;
                            }

                            let field_name = &field.ident;

                            pattern_matches.push(quote! { ref #field_name });

                            fields_quoted.append(quote! {
                                + _serde_mtproto::MtProtoSized::size_hint(#field_name)?
                            });
                        }

                        pattern_match_quoted = quote! {
                            { #(#pattern_matches),* }
                        }
                    }

                    syn::VariantData::Tuple(ref fields) => {
                        let mut pattern_matches = Vec::new();

                        for (i, field) in fields.iter().enumerate() {
                            if is_skippable_field(field) {
                                continue;
                            }

                            let field_name = syn::Ident::new(format!("__field_{}", i));

                            pattern_matches.push(quote! { ref #field_name });

                            fields_quoted.append(quote! {
                                + _serde_mtproto::MtProtoSized::size_hint(#field_name)?
                            });
                        }

                        pattern_match_quoted = quote! {
                            (#(#pattern_matches),*)
                        }
                    }

                    syn::VariantData::Unit => {
                        pattern_match_quoted = quote! {};
                    }
                }

                variants_quoted.append(quote! {
                    #item_name::#variant_name #pattern_match_quoted => {
                        Ok(#fields_quoted)
                    },
                });
            }

            quote! {
                match *self {
                    #variants_quoted
                }
            }
        }
    };

    quote! {
        #[allow(non_upper_case_globals)]
        const #dummy_const: () = {
            extern crate serde_mtproto as _serde_mtproto;

            impl #item_impl_generics _serde_mtproto::MtProtoSized for #item_name #item_ty_generics
                #item_where_clause
            {
                fn size_hint(&self) -> _serde_mtproto::Result<usize> {
                    #size_hint_body
                }
            }
        };
    }
}

fn add_mt_proto_sized_trait_bound_if_missing(ast: &mut syn::DeriveInput) {
    'ty_param: for ty_param in &mut ast.generics.ty_params {
        for bound in &ty_param.bounds {
            match *bound {
                syn::TyParamBound::Trait(ref poly_trait_ref, syn::TraitBoundModifier::None) => {
                    let path = &poly_trait_ref.trait_ref;
                    if path.global {
                        continue;
                    }

                    let trait_ref_segments = path.segments
                        .iter()
                        .map(|s| s.ident.as_ref());
                    let mt_proto_sized_segments = vec!["_serde_mtproto", "MtProtoSized"].into_iter();

                    if trait_ref_segments.eq(mt_proto_sized_segments) {
                        continue 'ty_param;
                    }
                },
                _ => (),
            }
        }

        ty_param.bounds.push(syn::TyParamBound::Trait(
            syn::PolyTraitRef {
                bound_lifetimes: vec![],
                trait_ref: syn::Path {
                    global: false,
                    segments: vec!["_serde_mtproto".into(), "MtProtoSized".into()],
                }
            },
            syn::TraitBoundModifier::None,
        ));
    }
}

fn is_skippable_field(field: &syn::Field) -> bool {
    for attr in &field.attrs {
        if let syn::Attribute {
            style: syn::AttrStyle::Outer,
            value: syn::MetaItem::List(ref namespace_ident, ref nested_meta_items),
            is_sugared_doc: false,
        } = *attr {
            if namespace_ident.as_ref() == "mtproto_sized" {
                for nested_mi in nested_meta_items {
                    if let syn::NestedMetaItem::MetaItem(ref meta_item) = *nested_mi {
                        if let syn::MetaItem::Word(ref flag_ident) = *meta_item {
                            if flag_ident == "skip" {
                                return true;
                            }
                        }
                    }
                }
            }
        }
    }

    false
}
