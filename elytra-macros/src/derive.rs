use proc_macro::TokenStream;

pub fn derive_elytra_proto(_input: TokenStream) -> TokenStream {
    TokenStream::new()
}

pub fn section(_input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    annotated_item
}