use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, LitStr, FnArg};

#[proc_macro_attribute]
pub fn template(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let template_path = parse_macro_input!(attr as LitStr);
    
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_generics = &input_fn.sig.generics;
    let fn_inputs = &input_fn.sig.inputs;
    let block = &input_fn.block;
    
    // Extract argument names for passing to inner function
    let arg_names = fn_inputs.iter().map(|arg| {
        match arg {
            FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(pat_ident) = &*pat_type.pat {
                    &pat_ident.ident
                } else {
                    panic!("Unsupported argument pattern")
                }
            }
            FnArg::Receiver(_) => panic!("Self arguments are not supported"),
        }
    });
    
    let expanded = quote! {
        // The actual function remains private
        async fn __inner_#fn_name #fn_generics(#fn_inputs) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
            #block
        }

        // The public wrapper function that users will call
        #fn_vis async fn #fn_name #fn_generics(#fn_inputs) -> Result<String, Box<dyn std::error::Error>> {
            // Get the template path relative to CARGO_MANIFEST_DIR
            let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")
                .map_err(|e| format!("Failed to get CARGO_MANIFEST_DIR: {}", e))?;
            let template_path = std::path::Path::new(&manifest_dir).join(#template_path_str);
            
            // Call internal macro function to handle template rendering
            my_template_macro::__private_render_template(
                &template_path,
                __inner_#fn_name(#(#arg_names),*).await?
            ).await
        }
    };

    TokenStream::from(expanded)
}