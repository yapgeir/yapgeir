use darling::{ast, util, FromDeriveInput, FromField};
use quote::{quote, ToTokens};

#[derive(Debug, FromField)]
#[darling(attributes(vertex))]
struct VertexField {
    ident: Option<syn::Ident>,
    ty: syn::Type,

    #[darling(default)]
    name: Option<String>,
    #[darling(default)]
    ignore: bool,
}

#[derive(Debug, FromDeriveInput)]
#[darling(supports(struct_named))]
pub struct Vertex {
    ident: syn::Ident,
    generics: syn::Generics,
    data: ast::Data<util::Ignored, VertexField>,
}

impl ToTokens for Vertex {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Vertex {
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

                // Attribute name is taken from the macro #[vertex(name)] attribute if it's defined,
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
                    yapgeir_graphics_hal::vertex_buffer::VertexAttribute {
                        name: #name,
                        offset: #offset,
                        kind: <#field_ty as yapgeir_graphics_hal::vertex_buffer::AsAttributeKind>::KIND,
                        size: <#field_ty as yapgeir_graphics_hal::vertex_buffer::AsAttributeKind>::SIZE
                    }
                })
            });

        tokens.extend(quote! {
            impl #imp yapgeir_graphics_hal::vertex_buffer::Vertex for #ident #ty #wher {
                const FORMAT: &'static [yapgeir_graphics_hal::vertex_buffer::VertexAttribute] = &[
                    #(#attributes,)*
                ];
            }
        });
    }
}
