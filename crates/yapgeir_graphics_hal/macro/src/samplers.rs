use darling::{ast, util, FromDeriveInput, FromField};
use quote::{quote, ToTokens};
use syn::punctuated::Punctuated;

#[derive(Debug, FromField)]
#[darling(attributes(sampler))]
struct SamplersField {
    ident: Option<syn::Ident>,

    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    location: Option<usize>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(samplers), supports(struct_named))]
pub struct Samplers {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<util::Ignored, SamplersField>,

    #[darling(default)]
    context: Option<String>,
}

impl ToTokens for Samplers {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Samplers {
            ref ident,
            ref generics,
            ref data,
            ref context,
        } = *self;

        let imp = generics.type_params();
        let (_, ty, wher) = generics.split_for_impl();

        let lt = format!("'{}_self_lifetime", ident);
        let lt = syn::Lifetime::new(&lt, proc_macro2::Span::call_site());

        let fields = data.as_ref().take_struct().unwrap().fields;
        let count = fields.len();

        let samplers = fields.iter().enumerate().filter_map(|(location, field)| {
            let field_ident = field.ident.as_ref().unwrap();

            // Texture name is taken from the macro #[sampler(name)] attribute if it's defined,
            // and defaulted to a field name.
            let name = match &field.name {
                Some(name) => quote!(#name),
                None => {
                    let name = format!("{}", field_ident);
                    quote!(#name)
                }
            };

            let location = field.location.unwrap_or(location) as u8;

            Some(quote! {
                yapgeir_graphics_hal::samplers::SamplerAttribute {
                    name: #name,
                    location: #location,
                    sampler: self.#field_ident.as_borrowed(),
                }
            })
        });

        // Try to find a generic parameter that implements Graphics trait and use that for this macro.
        // Otherwise force user to provide the context.
        let trait_bounds = &["yapgeir_graphics_hal::Graphics", "Graphics"];
        let context = match find_generic_implementing_trait(generics, trait_bounds) {
            Some(context) => context,
            None => {
                let context = context
                    .as_ref()
                    .expect("Context attribute must be provided: #[samplers(context = TYPE)]");
                syn::parse_str::<syn::Type>(context).unwrap()
            }
        };

        tokens.extend(quote! {
            impl <#(#imp),*> yapgeir_graphics_hal::samplers::Samplers<#context, #count> for #ident #ty #wher {
                fn attributes<#lt>(&#lt self) -> [yapgeir_graphics_hal::samplers::SamplerAttribute<#context, &#lt #context::Texture>; #count] {
                    [#(#samplers,)*]
                }
            }
        });
    }
}

fn find_generic_implementing_trait<'a>(
    generics: &'a syn::Generics,
    trait_bounds: &[&str],
) -> Option<syn::Type> {
    for param in &generics.params {
        if let syn::GenericParam::Type(t) = param {
            if has_trait_bounds(&t.bounds, trait_bounds) {
                return Some(syn::Type::Path(syn::TypePath {
                    qself: None,
                    path: t.ident.clone().into(),
                }));
            }
        }
    }

    if let Some(where_clause) = &generics.where_clause {
        for predicate in &where_clause.predicates {
            if let syn::WherePredicate::Type(t) = predicate {
                if has_trait_bounds(&t.bounds, trait_bounds) {
                    return Some(t.bounded_ty.clone());
                }
            }
        }
    }

    None
}

fn has_trait_bounds<T>(bounds: &Punctuated<syn::TypeParamBound, T>, trait_bounds: &[&str]) -> bool {
    bounds
        .iter()
        .filter_map(|bound| match bound {
            syn::TypeParamBound::Trait(trait_bound) => Some(&trait_bound.path),
            _ => None,
        })
        .any(|path| trait_bounds.iter().any(|b| path.is_ident(b)))
}
