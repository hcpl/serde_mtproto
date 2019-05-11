macro_rules! ident {
    ($($format_args:tt)*) => {
        proc_macro2::Ident::new(&std::format!($($format_args)*), proc_macro2::Span::call_site())
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
    ($expr:expr=> ) => { proc_macro2::TokenStream::new() };
    ($expr:expr=> $($tt:tt)+) => {{
        use std::iter::FromIterator;

        let mut span_tokens = quote::ToTokens::into_token_stream($expr).into_iter();
        let start_span = span_tokens.next().map_or_else(proc_macro2::Span::call_site, |t| t.span());
        let end_span = span_tokens.last().map_or(start_span, |t| t.span());

        let mut end_tokens = quote::quote_spanned!(end_span=> $($tt)*).into_iter();
        let mut start_token = end_tokens.next().unwrap();
        start_token.set_span(start_span);

        proc_macro2::TokenStream::from_iter(std::iter::once(start_token).chain(end_tokens))
    }};
}

// Adapted from `if_chain!` macro from `if_chain` 0.1.3
macro_rules! control_flow_chain {
    ($($tt:tt)*) => {
        __control_flow_chain! { @init () $($tt)* }
    };
}

macro_rules! __control_flow_chain {
    // Expand with both a successful case and a fallback
    (@init ($($tt:tt)*) then { $($then:tt)* } else { $($other:tt)* }) => {
        __control_flow_chain! { @expand { $($other)* } $($tt)* then { $($then)* } }
    };
    // Expand with no fallback
    (@init ($($tt:tt)*) then { $($then:tt)* }) => {
        __control_flow_chain! { @expand {} $($tt)* then { $($then)* } }
    };
    // Munch everything until either of the arms above can be matched.
    // Munched tokens are placed into `$($tt)*`
    (@init ($($tt:tt)*) $head:tt $($tail:tt)*) => {
        __control_flow_chain! { @init ($($tt)* $head) $($tail)* }
    };

    // `let` with single pattern
    (@expand { $($other:tt)* } let $pat:pat = $expr:expr; $($tt:tt)+) => {
        {
            let $pat = $expr;
            __control_flow_chain! { @expand { $($other)* } $($tt)+ }
        }
    };
    // `let` with multiple patterns
    (@expand { $($other:tt)* } let $pat1:pat | $($pat:pat)|+ = $expr:expr; $($tt:tt)+) => {
        match $expr {
            $pat1 | $($pat)|+ => __control_flow_chain! { @expand { $($other)* } $($tt)+ }
        }
    };
    // `if let` with single pattern
    (@expand {} if let $pat:pat = $expr:expr; $($tt:tt)+) => {
        if let $pat = $expr {
            __control_flow_chain! { @expand {} $($tt)+ }
        }
    };
    // `if let` with single pattern and a fallback
    (@expand { $($other:tt)+ } if let $pat:pat = $expr:expr; $($tt:tt)+) => {
        if let $pat = $expr {
            __control_flow_chain! { @expand { $($other)+ } $($tt)+ }
        } else {
            $($other)+
        }
    };
    // `if let` with multiple matterns and a fallback (if present)
    (@expand { $($other:tt)* } if let $pat1:pat | $($pat:pat)|+ = $expr:expr; $($tt:tt)+) => {
        match $expr {
            $pat1 | $($pat)|+ => { __control_flow_chain! { @expand { $($other)* } $($tt)+ } },
            _ => { $($other)* }
        }
    };
    // `if` with a successful case
    (@expand {} if $expr:expr; $($tt:tt)+) => {
        if $expr {
            __control_flow_chain! { @expand {} $($tt)+ }
        }
    };
    // `if` with both a successful case and a fallback
    (@expand { $($other:tt)+ } if $expr:expr; $($tt:tt)+) => {
        if $expr {
            __control_flow_chain! { @expand { $($other)+ } $($tt)+ }
        } else {
            $($other)+
        }
    };
    // `for` loop
    // TODO: how do loops deal with fallbacks?
    (@expand { $($other:tt)* } for $elem:pat in $iter:expr; $($tt:tt)+) => {
        for $elem in $iter {
            __control_flow_chain! { @expand { $($other)* } $($tt)+ }
        }
    };
    // Final macro call
    (@expand { $($other:tt)* } then { $($then:tt)* }) => {
        $($then)*
    };
}
