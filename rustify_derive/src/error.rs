use proc_macro2::Span;

/// The general error object returned by functions in this crate.
///
/// The error object can be directly converted from a [syn::Error] as well as
/// be converted directly into a [proc_macro2::TokenStream] to be returned to
/// the compiler.
#[derive(Debug)]
pub struct Error(proc_macro2::TokenStream);

impl Error {
    /// Returns a new instance of [Error] using the given [Span] and message.
    ///
    /// This uses [quote_spanned!] in order to provide more accurate information
    /// to the compiler about the exact location of the error.
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
