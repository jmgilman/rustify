use proc_macro2::Span;

#[derive(Debug)]
pub struct Error(proc_macro2::TokenStream);

impl Error {
    pub fn new(span: Span, message: &str) -> Error {
        Error(quote_spanned! { span =>
            compile_error!(#message);
        })
    }

    pub fn into_tokens(self) -> proc_macro2::TokenStream {
        self.0
    }
}

impl From<syn::Error> for Error {
    fn from(e: syn::Error) -> Error {
        Error(e.to_compile_error())
    }
}
