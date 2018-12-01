use proc_macro2;
use syn;

use ast;


pub(crate) fn impl_mt_proto_sized(mut container: ast::Container) -> proc_macro2::TokenStream {
    add_mt_proto_sized_trait_bound_if_missing(&mut container);
    let (item_impl_generics, item_ty_generics, item_where_clause) =
        container.generics.split_for_impl();

    let item_name = &container.ident;
    let dummy_const = ident!("_IMPL_MT_PROTO_SIZED_FOR__{}", item_name);

    let size_hint_body = match container.data {
        ast::Data::Struct(ref data_struct) => {
            match data_struct.fields {
                syn::Fields::Named(ref fields) => {
                    let size_hints = fields.named.iter().filter_map(|field| {
                        if is_skippable_field(field) {
                            return None;
                        }

                        let field_name = &field.ident;
                        let func = quote_spanned_by! {field=>
                            _serde_mtproto::MtProtoSized::size_hint
                        };

                        Some(quote!(#func(&self.#field_name)?))
                    });

                    quote!(Ok(0 #(+ #size_hints)*))
                },
                syn::Fields::Unnamed(ref fields) => {
                    let size_hints = fields.unnamed.iter().enumerate().filter_map(|(i, field)| {
                        if is_skippable_field(field) {
                            return None;
                        }

                        // Integers are rendered with type suffixes. We don't want this.
                        let field_index = syn::Index::from(i);
                        let func = quote_spanned_by! {field=>
                            _serde_mtproto::MtProtoSized::size_hint
                        };

                        Some(quote!(#func(&self.#field_index)?))
                    });

                    quote!(Ok(0 #(+ #size_hints)*))
                },
                syn::Fields::Unit => quote!(Ok(0)),
            }
        },
        ast::Data::Enum(ref data_enum) => {
            let variants_quoted = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                match variant.fields {
                    syn::Fields::Named(ref fields) => {
                        let (patterns, size_hints) = fields.named.iter().filter_map(|field| {
                            if is_skippable_field(field) {
                                return None;
                            }

                            let field_name = &field.ident;
                            let func = quote_spanned_by! {field=>
                                _serde_mtproto::MtProtoSized::size_hint
                            };

                            let pattern = quote!(ref #field_name);
                            let size_hint = quote!(#func(#field_name)?);

                            Some((pattern, size_hint))
                        }).unzip::<_, _, Vec<_>, Vec<_>>();

                        quote! {
                            #item_name::#variant_name { #(#patterns),* } => {
                                Ok(0 #(+ #size_hints)*)
                            }
                        }
                    },
                    syn::Fields::Unnamed(ref fields) => {
                        let (patterns, size_hints) = fields.unnamed.iter().enumerate()
                            .filter_map(|(i, field)|
                        {
                            if is_skippable_field(field) {
                                return None;
                            }

                            let field_name = ident!("__field_{}", i);
                            let func = quote_spanned_by! {field=>
                                _serde_mtproto::MtProtoSized::size_hint
                            };

                            let pattern = quote!(ref #field_name);
                            let size_hint = quote!(#func(#field_name)?);

                            Some((pattern, size_hint))
                        }).unzip::<_, _, Vec<_>, Vec<_>>();

                        quote! {
                            #item_name::#variant_name(#(#patterns),*) => {
                                Ok(0 #(+ #size_hints)*)
                            }
                        }
                    },
                    syn::Fields::Unit => {
                        quote! {
                            #item_name::#variant_name => Ok(0),
                        }
                    },
                }
            });

            quote! {
                match *self {
                    #(#variants_quoted)*
                }
            }
        },
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


fn add_mt_proto_sized_trait_bound_if_missing(container: &mut ast::Container) {
    'param: for param in &mut container.generics.params {
        if let syn::GenericParam::Type(ref mut type_param) = *param {
            for bound in &type_param.bounds {
                if let syn::TypeParamBound::Trait(ref trait_bound) = *bound {
                    if let syn::TraitBoundModifier::None = trait_bound.modifier {
                        continue;
                    }

                    let path = &trait_bound.path;
                    if path.leading_colon.is_some() {
                        continue;
                    }

                    let trait_ref_segments = path.segments
                        .iter()
                        .map(|s| s.ident.to_string());
                    let mt_proto_sized_segments = vec!["_serde_mtproto", "MtProtoSized"].into_iter();

                    if trait_ref_segments.eq(mt_proto_sized_segments) {
                        continue 'param;
                    }
                }
            }

            type_param.bounds.push(parse_quote!(_serde_mtproto::MtProtoSized));
        }
    }
}

fn is_skippable_field(field: &syn::Field) -> bool {
    for attr in &field.attrs {
        if let syn::AttrStyle::Inner(..) = attr.style {
            continue;
        }

        if let Some(syn::Meta::List(list)) = attr.interpret_meta() {
            if list.ident == "mtproto_sized" {
                for nested_meta in list.nested {
                    if let syn::NestedMeta::Meta(syn::Meta::Word(ident)) = nested_meta {
                        if ident == "skip" {
                            return true;
                        }
                    }
                }
            }
        }
    }

    false
}
