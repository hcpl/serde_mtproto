use proc_macro2;
use quote;
use syn;


pub(crate) struct Container {
    pub(crate) attrs: Vec<syn::Attribute>,
    pub(crate) ident: proc_macro2::Ident,
    pub(crate) generics: syn::Generics,
    pub(crate) data: Data,
}

pub(crate) enum Data {
    Struct(syn::DataStruct),
    Enum(syn::DataEnum),
}

impl Container {
    pub(crate) fn from_derive_input(input: syn::DeriveInput) -> Option<Self> {
        let data = match input.data {
            syn::Data::Struct(data_struct) => Data::Struct(data_struct),
            syn::Data::Enum(data_enum) => Data::Enum(data_enum),
            syn::Data::Union(_) => return None,
        };

        Some(Self {
            attrs: input.attrs,
            ident: input.ident,
            generics: input.generics,
            data,
        })
    }
}


// A helper to exclude attributes from spans
pub(crate) struct FieldNoAttrs<'a> {
    pub(crate) vis: &'a syn::Visibility,
    pub(crate) ident: &'a Option<syn::Ident>,
    pub(crate) colon_token: &'a Option<Token![:]>,
    pub(crate) ty: &'a syn::Type,
}

impl<'a> FieldNoAttrs<'a> {
    pub(crate) fn from_field(field: &'a syn::Field) -> Self {
        Self {
            vis: &field.vis,
            ident: &field.ident,
            colon_token: &field.colon_token,
            ty: &field.ty,
        }
    }
}

impl<'a> quote::ToTokens for FieldNoAttrs<'a> {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.vis.to_tokens(tokens);
        if let Some(ref ident) = *self.ident {
            ident.to_tokens(tokens);
            match *self.colon_token {
                Some(ref colon_token) => colon_token.to_tokens(tokens),
                None => <Token![:]>::default().to_tokens(tokens),
            }
        }
        self.ty.to_tokens(tokens);
    }
}
