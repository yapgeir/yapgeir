use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{parse_quote, spanned::Spanned, ItemFn};

pub fn gen_instrument(input: TokenStream) -> Result<TokenStream, String> {
    let mut fn_block = match syn::parse2::<ItemFn>(input) {
        Ok(impl_block) => impl_block,
        Err(_) => return Err("#[instrument] only applies to system functions".into()),
    };

    fn_block.attrs.clear();

    let instr_ident = Ident::new("___instrumentation___", fn_block.span());

    fn_block.sig.inputs.push(parse_quote! {
        mut #instr_ident: std::option::Option<yapgeir_realm::ResMut<yapgeir_instrument::Instrumentation>>
    });

    let fn_name = &fn_block.sig.ident;
    fn_block.block.stmts.insert(
        0,
        parse_quote! {
            let _instr = #instr_ident.as_mut().map(|i| i.guard(concat!(module_path!(), "::", stringify!(#fn_name))));
        },
    );

    Ok(quote! {
       #fn_block
    })
}
