use darling::util::parse_expr;
use darling::{ast, util, FromDeriveInput, FromField};
use quote::{quote, ToTokens};
use syn::{parse_quote, Expr};

#[derive(Debug, FromField)]
#[darling(attributes(slot_map))]
struct PersistentSlotMapKeysField {
    ident: Option<syn::Ident>,
    #[darling(with = parse_expr::preserve_str_literal, map = Some)]
    slot: Option<Expr>,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named), attributes(slot_map))]
pub struct PersistentSlotMapKeys {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<util::Ignored, PersistentSlotMapKeysField>,

    key_type: Option<syn::Path>,
}

impl ToTokens for PersistentSlotMapKeys {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let PersistentSlotMapKeys {
            ref ident,
            ref generics,
            ref data,
            ref key_type,
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        let fields = data.as_ref().take_struct().unwrap().fields;

        let key_ty = key_type
            .to_owned()
            .unwrap_or_else(|| parse_quote!(std::string::String));

        let fields = fields.iter().filter_map(|field| {
            let field_ident = field.ident.as_ref().unwrap();

            let key = match &field.slot {
                Some(name) => name.to_owned(),
                None => {
                    let slot = format!("{}", field_ident);
                    parse_quote!(#slot)
                }
            };

            let err = format!(
                "Key {} not found in PersistentSlotMap",
                key.to_token_stream()
            );

            Some(quote! {
                #field_ident: slot_map.find_slot_by_key(#key)
                    .expect(#err)
                    .into()
            })
        });

        tokens.extend(quote! {
            impl #imp yapgeir_collections::PersistentSlotMapKeys<#key_ty> for #ident #ty #wher {
                fn new<V>(slot_map: &yapgeir_collections::PersistentSlotMap<#key_ty, V>) -> Self {
                    Self {
                        #(#fields,)*
                    }
                }
            }
        });
    }
}
