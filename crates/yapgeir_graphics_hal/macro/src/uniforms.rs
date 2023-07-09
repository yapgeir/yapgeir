use darling::{ast, util, FromDeriveInput, FromField};
use quote::{quote, ToTokens};

#[derive(Debug, FromField)]
#[darling(attributes(uniforms))]
struct UniformsField {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    ignore: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
pub struct Uniforms {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<util::Ignored, UniformsField>,
}

impl ToTokens for Uniforms {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Uniforms {
            ref ident,
            ref generics,
            ref data,
        } = *self;

        let (imp, ty, wher) = generics.split_for_impl();

        let fields = data.as_ref().take_struct().unwrap().fields;

        let attributes = fields
            .iter()
            .filter(|field| !field.ignore)
            .filter_map(|field| {
                let field_ident = field.ident.as_ref().unwrap();
                let field_ty = &field.ty;

                // Attribute name is taken from the macro #[uniforms(name)] attribute if it's defined,
                // and defaulted to a field name.
                let name = match &field.name {
                    Some(name) => quote!(#name),
                    None => {
                        let name = format!("{}", field_ident);
                        // Padding fields have names starting with __, ignore them
                        if name.starts_with("__") {
                            return None;
                        }
                        quote!(#name)
                    }
                };

                // Macro will expand offset to this block.
                // Luckily it's const, so it will be inlined into a usize during compilation
                let offset = quote! {
                    {
                        let uninit = core::mem::MaybeUninit::<#ident>::uninit();
                        let uninit_ptr = uninit.as_ptr();
                        let field_ptr = unsafe { &(*uninit_ptr).#field_ident as *const _ };

                        unsafe { (field_ptr as *const u8).offset_from(uninit_ptr as *const u8) as usize }
                    }
                };

                Some(quote! {
                    yapgeir_graphics_hal::uniforms::UniformAttribute {
                        name: #name,
                        offset: #offset,
                        size: std::mem::size_of::<#field_ty>(),
                    }
                })
            });

        tokens.extend(quote! {
            impl #imp yapgeir_graphics_hal::uniforms::Uniforms for #ident #ty #wher {
                const FORMAT: &'static [yapgeir_graphics_hal::uniforms::UniformAttribute] = &[
                    #(#attributes,)*
                ];
            }
        });
    }
}
