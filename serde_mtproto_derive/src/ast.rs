use proc_macro2;
use quote;
use syn;


pub(crate) struct Container {
    pub(crate) attrs: Vec<syn::Attribute>,
    pub(crate) vis: syn::Visibility,
    pub(crate) ident: proc_macro2::Ident,
    pub(crate) generics: syn::Generics,
    pub(crate) data: Data,
}

pub(crate) enum Data {
    Struct(syn::DataStruct),
    Enum(syn::DataEnum),
}

impl Container {
    pub(crate) fn from_derive_input(input: syn::DeriveInput, trait_name: &str) -> syn::Result<Self> {
        let data = match input.data {
            syn::Data::Struct(data_struct) => Data::Struct(data_struct),
            syn::Data::Enum(data_enum) => Data::Enum(data_enum),
            syn::Data::Union(_) => {
                let msg = format!("Cannot derive `{}` for unions", trait_name);
                return Err(syn::Error::new_spanned(input, msg));
            },
        };

        let syn::DeriveInput { attrs, vis, ident, generics, data: _ } = input;

        Ok(Self { attrs, vis, ident, generics, data })
    }
}

impl quote::ToTokens for Container {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        for attr in self.attrs.iter().filter(|a| matches!(a.style, syn::AttrStyle::Outer)) {
            attr.to_tokens(tokens);
        }
        self.vis.to_tokens(tokens);
        match self.data {
            Data::Struct(ref d) => d.struct_token.to_tokens(tokens),
            Data::Enum(ref d) => d.enum_token.to_tokens(tokens),
        }
        self.ident.to_tokens(tokens);
        self.generics.to_tokens(tokens);
        match self.data {
            Data::Struct(ref data) => match data.fields {
                syn::Fields::Named(ref fields) => {
                    self.generics.where_clause.to_tokens(tokens);
                    fields.to_tokens(tokens);
                },
                syn::Fields::Unnamed(ref fields) => {
                    fields.to_tokens(tokens);
                    self.generics.where_clause.to_tokens(tokens);
                    TokensOrDefault(&data.semi_token).to_tokens(tokens);

                },
                syn::Fields::Unit => {
                    self.generics.where_clause.to_tokens(tokens);
                    TokensOrDefault(&data.semi_token).to_tokens(tokens);
                },
            },
            Data::Enum(ref data) => {
                self.generics.where_clause.to_tokens(tokens);
                data.brace_token.surround(tokens, |tokens| {
                    data.variants.to_tokens(tokens);
                });
            },
        }
    }
}


pub(crate) struct TokensOrDefault<'a, T: 'a>(pub(crate) &'a Option<T>);

impl<'a, T> quote::ToTokens for TokensOrDefault<'a, T>
where
    T: quote::ToTokens + Default,
{
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match *self.0 {
            Some(ref t) => t.to_tokens(tokens),
            None => T::default().to_tokens(tokens),
        }
    }
}
