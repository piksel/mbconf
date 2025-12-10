use proc_macro::TokenStream;

mod derive;

#[proc_macro_derive(Proto, attributes(section))]
pub fn derive_elytra_proto(input: TokenStream) -> TokenStream {
    derive::derive_elytra_proto(input)
}

#[proc_macro_attribute]
pub fn section(input: TokenStream, annotated_item: TokenStream) -> TokenStream {
    derive::section(input, annotated_item)
}