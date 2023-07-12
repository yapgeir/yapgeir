use darling::FromDeriveInput;
use persistent_slot_map_keys::PersistentSlotMapKeys;
use proc_macro::TokenStream;
use quote::quote;

mod persistent_slot_map_keys;

#[proc_macro_derive(PersistentSlotMapKeys, attributes(slot_map))]
pub fn from_indexed_map_derive(input: TokenStream) -> TokenStream {
    let input = syn::parse2(input.into()).unwrap();
    let output = PersistentSlotMapKeys::from_derive_input(&input).unwrap();
    quote!(#output).into()
}
