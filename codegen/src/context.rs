// use proc_macro2::TokenStream;
// use quote::quote;
// use syn::{parse_macro_input, DeriveInput};

// #[proc_macro_derive(Context)]
// pub fn derive_context(input: TokenStream) -> TokenStream {
//     let input = parse_macro_input!(input as DeriveInput);
//     let name = input.ident;

//     let expanded = quote! {
//         impl Context for #name {}
//     };

//     TokenStream::from(expanded)
// }