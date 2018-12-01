macro_rules! ident {
    ($($format_args:tt)*) => {
        ::proc_macro2::Ident::new(&format!($($format_args)*), ::proc_macro2::Span::call_site())
    };
}

macro_rules! matches {
    ($expr:expr, $($pat:tt)+) => {
        match $expr {
            $($pat)+ => true,
            _        => false,
        }
    };
}

macro_rules! quote_spanned_by {
    ($expr:expr=> ) => { ::proc_macro2::TokenStream::new() };
    ($expr:expr=> $($tt:tt)+) => {{
        use std::iter::FromIterator;

        let mut span_tokens = ::quote::ToTokens::into_token_stream($expr).into_iter();
        let start_span = span_tokens.next().map_or_else(::proc_macro2::Span::call_site, |t| t.span());
        let end_span = span_tokens.last().map_or(start_span, |t| t.span());

        let mut end_tokens = quote_spanned!(end_span=> $($tt)*).into_iter();
        let mut start_token = end_tokens.next().unwrap();
        start_token.set_span(start_span);

        ::proc_macro2::TokenStream::from_iter(::std::iter::once(start_token).chain(end_tokens))
    }};
}
