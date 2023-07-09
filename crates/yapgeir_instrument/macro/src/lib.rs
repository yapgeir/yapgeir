use instrument::gen_instrument;
use proc_macro::TokenStream;

mod instrument;

#[proc_macro_attribute]
pub fn instrument(_args: TokenStream, input: TokenStream) -> TokenStream {
    gen_instrument(input.into())
        .expect("Unable to instrument system")
        .into()
}
