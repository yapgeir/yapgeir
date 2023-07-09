use proc_macro::TokenStream;
use system::gen_system;

mod system;

#[proc_macro_attribute]
pub fn system(_args: TokenStream, input: TokenStream) -> TokenStream {
    gen_system(input.into())
        .expect("Failed to generate fluent methods")
        .into()
}
