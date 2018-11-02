use proc_macro2::{self, Span};
use syn::{
    AttrStyle, Data, DeriveInput, Field, Fields, GenericParam, Ident, Index,
    Meta, NestedMeta, TraitBoundModifier, TypeParamBound,
};
use syn::spanned::Spanned;


pub fn impl_mt_proto_sized(ast: &mut DeriveInput) -> proc_macro2::TokenStream {
    add_mt_proto_sized_trait_bound_if_missing(ast);
    let (item_impl_generics, item_ty_generics, item_where_clause) = ast.generics.split_for_impl();

    let item_name = &ast.ident;
    let dummy_const =
        Ident::new(&format!("_IMPL_MT_PROTO_SIZED_FOR_{}", item_name), Span::call_site());

    let size_hint_body = match ast.data {
        Data::Struct(ref data_struct) => {
            match data_struct.fields {
                Fields::Named(ref fields) => {
                    let size_hints = fields.named.iter().filter_map(|field| {
                        if is_skippable_field(field) {
                            return None;
                        }

                        let field_name = &field.ident;
                        let func = quote_spanned! {field.span()=>
                            _serde_mtproto::MtProtoSized::size_hint
                        };

                        Some(quote!(#func(&self.#field_name)?))
                    });

                    quote!(Ok(0 #(+ #size_hints)*))
                },
                Fields::Unnamed(ref fields) => {
                    let size_hints = fields.unnamed.iter().enumerate().filter_map(|(i, field)| {
                        if is_skippable_field(field) {
                            return None;
                        }

                        // Integers are rendered with type suffixes. We don't want this.
                        let field_index = Index::from(i);
                        let func = quote_spanned! {field.span()=>
                            _serde_mtproto::MtProtoSized::size_hint
                        };

                        Some(quote!(#func(&self.#field_index)?))
                    });

                    quote!(Ok(0 #(+ #size_hints)*))
                },
                Fields::Unit => quote! { Ok(0) },
            }
        },
        Data::Enum(ref data_enum) => {
            let variants_quoted = data_enum.variants.iter().map(|variant| {
                let variant_name = &variant.ident;

                match variant.fields {
                    Fields::Named(ref fields) => {
                        let (patterns, size_hints) = fields.named.iter().filter_map(|field| {
                            if is_skippable_field(field) {
                                return None;
                            }

                            let field_name = &field.ident;
                            let func = quote_spanned! {field.span()=>
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
                    Fields::Unnamed(ref fields) => {
                        let (patterns, size_hints) = fields.unnamed.iter().enumerate()
                            .filter_map(|(i, field)|
                        {
                            if is_skippable_field(field) {
                                return None;
                            }

                            let field_name = Ident::new(&format!("__field_{}", i), Span::call_site());
                            let func = quote_spanned! {field.span()=>
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
                    Fields::Unit => {
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
        Data::Union(_) => panic!("Cannot derive `mtproto::Identifiable` for unions."),
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


fn add_mt_proto_sized_trait_bound_if_missing(ast: &mut DeriveInput) {
    'param: for param in &mut ast.generics.params {
        if let GenericParam::Type(ref mut type_param) = *param {
            for bound in &type_param.bounds {
                if let TypeParamBound::Trait(ref trait_bound) = *bound {
                    if let TraitBoundModifier::None = trait_bound.modifier {
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

fn is_skippable_field(field: &Field) -> bool {
    for attr in &field.attrs {
        if let AttrStyle::Inner(..) = attr.style {
            continue;
        }

        if let Some(Meta::List(list)) = attr.interpret_meta() {
            if list.ident == "mtproto_sized" {
                for nested_meta in list.nested {
                    if let NestedMeta::Meta(Meta::Word(ident)) = nested_meta {
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
