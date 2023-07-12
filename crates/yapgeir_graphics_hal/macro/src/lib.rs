use darling::FromDeriveInput;
use proc_macro::TokenStream;
use quote::quote;
use samplers::Samplers;
use uniforms::Uniforms;
use vertex::Vertex;

mod samplers;
mod uniforms;
mod vertex;

#[proc_macro_derive(Vertex, attributes(vertex))]
pub fn vertex_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse2(input.into()).unwrap();
    let output = Vertex::from_derive_input(&input).unwrap();
    quote!(#output).into()
}

#[proc_macro_derive(Uniforms, attributes(uniforms))]
pub fn uniforms_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse2(input.into()).unwrap();
    let output = Uniforms::from_derive_input(&input).unwrap();
    quote!(#output).into()
}

#[proc_macro_derive(Samplers, attributes(samplers))]
pub fn textures_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse2(input.into()).unwrap();
    let output = Samplers::from_derive_input(&input).unwrap();
    quote!(#output).into()
}
