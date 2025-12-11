use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

pub fn main_macro(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    let fn_name = &input_fn.sig.ident;
    let fn_block = &input_fn.block;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;

    let output = quote! {
        #(#fn_attrs)*
        #[cfg(not(target_arch = "wasm32"))]
        #fn_vis fn #fn_name() {
            #fn_block
        }

        #[cfg(target_arch = "wasm32")]
        #fn_vis fn #fn_name() {
            // Actual entry point is __gpui_wasm_main via wasm_bindgen(start)
        }

        #[cfg(target_arch = "wasm32")]
        #[wasm_bindgen::prelude::wasm_bindgen(start)]
        pub fn __gpui_wasm_main() {
            console_error_panic_hook::set_once();
            #fn_block
        }
    };

    output.into()
}
