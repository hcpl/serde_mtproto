use quote;
use syn::{Body, DeriveInput, Ident, VariantData};


pub fn impl_mt_proto_sized(ast: &DeriveInput) -> quote::Tokens {
    let item_name = &ast.ident;
    let (item_impl_generics, item_ty_generics, item_where_clause) = ast.generics.split_for_impl();

    let dummy_const = Ident::new(format!("_IMPL_MT_PROTO_SIZED_FOR_{}", item_name));

    let get_size_hint_body = match ast.body {
        Body::Struct(ref data) => {
            let mut fields_quoted = quote! { 0 };

            match *data {
                VariantData::Struct(ref fields) => {
                    for field in fields {
                        let field_name = &field.ident;

                        fields_quoted.append(quote! {
                            + _serde_mtproto::MtProtoSized::get_size_hint(&self.#field_name)?
                        });
                    }
                }

                VariantData::Tuple(ref fields) => {
                    for (i, _) in fields.iter().enumerate() {
                        fields_quoted.append(quote! {
                            + _serde_mtproto::MtProtoSized::get_size_hint(&self.#i)?
                        });
                    }
                }

                VariantData::Unit => {}
            }

            quote! {
                Ok(#fields_quoted)
            }
        }

        Body::Enum(ref variants) => {
            let mut variants_quoted = quote::Tokens::new();

            for variant in variants {
                let variant_name = &variant.ident;

                let pattern_match_quoted;
                let mut fields_quoted = quote! { 0 };

                match variant.data {
                    VariantData::Struct(ref fields) => {
                        let mut pattern_matches = Vec::new();

                        for field in fields {
                            let field_name = &field.ident;

                            pattern_matches.push(quote! { ref #field_name });

                            fields_quoted.append(quote! {
                                + _serde_mtproto::MtProtoSized::get_size_hint(#field_name)?
                            });
                        }

                        pattern_match_quoted = quote! {
                            { #(#pattern_matches),* }
                        }
                    }

                    VariantData::Tuple(ref fields) => {
                        let mut pattern_matches = Vec::new();

                        for (i, _) in fields.iter().enumerate() {
                            let field_name = Ident::new(format!("__field_{}", i));

                            pattern_matches.push(quote! { ref #field_name });

                            fields_quoted.append(quote! {
                                + _serde_mtproto::MtProtoSized::get_size_hint(#field_name)?
                            });
                        }

                        pattern_match_quoted = quote! {
                            (#(#pattern_matches),*)
                        }
                    }

                    VariantData::Unit => {
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
                fn get_size_hint(&self) -> _serde_mtproto::Result<usize> {
                    #get_size_hint_body
                }
            }
        };
    }
}
