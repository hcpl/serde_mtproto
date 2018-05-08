use std::iter::FromIterator;

use proc_macro2::Span;
use quote;
use syn::{
    Attribute, AttrStyle, Data, DeriveInput, Field, Fields, GenericParam, Ident, Index,
    Meta, NestedMeta, Path, PathSegment, TraitBound, TraitBoundModifier, TypeParamBound,
};


pub fn impl_mt_proto_sized(ast: &mut DeriveInput) -> quote::Tokens {
    add_mt_proto_sized_trait_bound_if_missing(ast);
    let (item_impl_generics, item_ty_generics, item_where_clause) = ast.generics.split_for_impl();

    let item_name = &ast.ident;
    let dummy_const =
        Ident::new(&format!("_IMPL_MT_PROTO_SIZED_FOR_{}", item_name), Span::call_site());

    let size_hint_body = match ast.data {
        Data::Struct(ref data_struct) => {
            let mut fields_quoted = quote! { 0 };

            match data_struct.fields {
                Fields::Named(ref fields_named) => {
                    for field_n in &fields_named.named {
                        if is_skippable_field(field_n) {
                            continue;
                        }

                        let field_name = &field_n.ident;

                        fields_quoted.append_all(&[quote! {
                            + _serde_mtproto::MtProtoSized::size_hint(&self.#field_name)?
                        }]);
                    }
                },
                Fields::Unnamed(ref fields_unnamed) => {
                    for (i, field_u) in fields_unnamed.unnamed.iter().enumerate() {
                        if is_skippable_field(field_u) {
                            continue;
                        }

                        assert!(i < u32::max_value() as usize);

                        // Integers are rendered with type suffixes. We don't want this.
                        let i = Index {
                            index: i as u32,
                            span: Span::call_site()
                        };

                        fields_quoted.append_all(&[quote! {
                            + _serde_mtproto::MtProtoSized::size_hint(&self.#i)?
                        }]);
                    }
                },
                Fields::Unit => (),
            }

            quote! { Ok(#fields_quoted) }
        },
        Data::Enum(ref data_enum) => {
            let mut variants_quoted = quote::Tokens::new();

            for variant in &data_enum.variants {
                let variant_name = &variant.ident;

                let pattern_match_quoted;
                let mut fields_quoted = quote! { 0 };

                match variant.fields {
                    Fields::Named(ref fields_named) => {
                        let mut pattern_matches = Vec::new();

                        for field_n in &fields_named.named {
                            if is_skippable_field(field_n) {
                                continue;
                            }

                            let field_name = &field_n.ident;

                            pattern_matches.push(quote! { ref #field_name });

                            fields_quoted.append_all(&[quote! {
                                + _serde_mtproto::MtProtoSized::size_hint(#field_name)?
                            }]);
                        }

                        pattern_match_quoted = quote! {
                            { #(#pattern_matches),* }
                        };
                    },
                    Fields::Unnamed(ref fields_unnamed) => {
                        let mut pattern_matches = Vec::new();

                        for (i, field_u) in fields_unnamed.unnamed.iter().enumerate() {
                            if is_skippable_field(field_u) {
                                continue;
                            }

                            let field_name = Ident::new(&format!("__field_{}", i), Span::call_site());

                            pattern_matches.push(quote! { ref #field_name });

                            fields_quoted.append_all(&[quote! {
                                + _serde_mtproto::MtProtoSized::size_hint(#field_name)?
                            }]);
                        }

                        pattern_match_quoted = quote! {
                            (#(#pattern_matches),*)
                        };
                    },
                    Fields::Unit => {
                        pattern_match_quoted = quote! {};
                    },
                }

                variants_quoted.append_all(&[quote! {
                    #item_name::#variant_name #pattern_match_quoted => {
                        Ok(#fields_quoted)
                    }
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
                    if path.global() {
                        continue;
                    }

                    let trait_ref_segments = path.segments
                        .iter()
                        .map(|s| s.ident.as_ref());
                    let mt_proto_sized_segments = vec!["_serde_mtproto", "MtProtoSized"].into_iter();

                    if trait_ref_segments.eq(mt_proto_sized_segments) {
                        continue 'param;
                    }
                }
            }

            type_param.bounds.push(TypeParamBound::Trait(TraitBound {
                paren_token: None,
                modifier: TraitBoundModifier::None,
                lifetimes: None,
                path: Path {
                    leading_colon: None,
                    segments: FromIterator::<PathSegment>::from_iter(vec!["_serde_mtproto".into(), "MtProtoSized".into()]),
                }
            }));
        }
    }
}

fn is_skippable_field(field: &Field) -> bool {
    for attr in &field.attrs {
        if let Attribute {
            style: AttrStyle::Outer,
            is_sugared_doc: false,
            ..
        } = *attr {
            if let Some(Meta::List(ref list)) = attr.interpret_meta() {
                if list.ident == "mtproto_sized" {
                    for nested_meta in &list.nested {
                        if let NestedMeta::Meta(ref meta) = *nested_meta {
                            if let Meta::Word(ref ident) = *meta {
                                if ident == "skip" {
                                    return true;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    false
}
