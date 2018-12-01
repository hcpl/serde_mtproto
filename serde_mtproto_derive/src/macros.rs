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
