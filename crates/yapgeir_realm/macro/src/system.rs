use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse_quote, FnArg, ItemImpl};

pub fn gen_system(input: TokenStream) -> Result<TokenStream, String> {
    let impl_block = match syn::parse2::<ItemImpl>(input) {
        Ok(impl_block) => impl_block,
        Err(_) => return Err("#[system] only applies to impl blocks".into()),
    };

    if impl_block.trait_.is_some() {
        Err("#[system] does not apply to trait impl blocks")?
    }

    // Find the method annotated with `system_run`
    let method = impl_block.items.iter().find_map(|item| match item {
        syn::ImplItem::Fn(method) => Some(method.clone()),
        _ => None,
    });

    let method = match method {
        Some(method) => method,
        None => Err("Could not find method annotated with `runner`")?,
    };

    let ty = &impl_block.self_ty;
    let delegate_name = &method.sig.ident;
    let (impl_generics, _, where_clause) = impl_block.generics.split_for_impl();

    let params = method
        .sig
        .inputs
        .iter()
        .filter_map(|i| match i {
            FnArg::Receiver(_) => None,
            FnArg::Typed(t) => Some(t),
        })
        .map(|_| {
            quote! {
                inject(resources)
            }
        });

    let rt = method.sig.output;
    let rtt = match &rt {
        syn::ReturnType::Default => parse_quote!(()),
        syn::ReturnType::Type(_, t) => *t.clone(),
    };

    let ty_str: proc_macro2::TokenStream = quote!(#ty);
    let trace = format!("::{}::{}", ty_str.to_string(), delegate_name.to_string());

    Ok(quote! {
        #impl_block

        impl #impl_generics ::yapgeir_realm::System<#rtt> for #ty #where_clause {
            fn run(&mut self, resources: &mut ::yapgeir_realm::Resources) #rt {
                use yapgeir_realm::SystemParam;

                fn inject<'a, T: yapgeir_realm::SystemParam<Item<'a> = T>>(
                    resources: &'a ::yapgeir_realm::Resources
                ) -> T {
                    match T::get(resources) {
                        Ok(t) => t,
                        Err(error) => panic!(
                            "Unable to inject resource into system {}. {}",
                            concat!(module_path!(), #trace), error
                        )
                    }
                }

                self.#delegate_name(#(#params,)*)
            }
        }
    })
}
