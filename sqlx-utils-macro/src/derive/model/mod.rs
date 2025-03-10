use proc_macro_error2::{abort, emit_error};
use proc_macro2::TokenStream;
use quote::{quote, quote_spanned, ToTokens};
use syn::{Field, Fields, Generics, Ident, spanned::Spanned};

#[cfg_attr(feature = "debug", derive(Debug))]
#[allow(dead_code)]
pub struct DeriveModel {
    struct_sig: syn::DataStruct,
    name: Ident,
    generics: Generics,
    id_field: Field,
}

impl DeriveModel {
    pub(crate) fn new(derive_input: syn::DeriveInput) -> Result<Self, TokenStream> {
        let span = derive_input.span();
        let name = derive_input.ident.clone();
        let generics = derive_input.generics.clone();

        let struct_sig = match derive_input.data {
            syn::Data::Struct(sig) => Ok(sig),
            syn::Data::Enum(..) => {
                Err(quote_spanned! { span => compile_error!("Expected Struct but found Enum") })
            }
            syn::Data::Union(..) => {
                Err(quote_spanned! { span => compile_error!("Expected Struct but found Enum") })
            }
        }?;

        let id_field = Self::find_id_field(&struct_sig.fields).unwrap_or_else(|| {
            abort!(
                name,
                "No field marked with #[model(id)] and no field named 'id' found";
                help = "Add #[model(id)] to the field that represents the model's ID"
            )
        });

        Ok(Self {
            struct_sig,
            name,
            generics,
            id_field,
        })
    }

    fn find_id_field(fields: &Fields) -> Option<Field> {
        // First, look for a field with #[model(id)] attribute
        for field in fields {
            if Self::has_id_attribute(field) {
                return Some(field.clone());
            }
        }

        // Fallback: look for a field named "id"
        for field in fields {
            if let Some(ident) = &field.ident {
                if ident == "id" {
                    return Some(field.clone());
                }
            }
        }

        None
    }

    fn has_id_attribute(field: &Field) -> bool {
        for attr in &field.attrs {
            if attr.path().is_ident("model") {
                let mut found = false;
                let result = attr.parse_nested_meta(|meta| {
                    if meta.path.is_ident("id") {
                        found = true;
                    }
                    Ok(())
                });

                if result.is_err() {
                    emit_error!(attr.span(), "Failed to parse model attribute");
                }

                if found {
                    return true;
                }
            }
        }
        false
    }

    fn expand(&self) -> TokenStream {
      let name = &self.name;
      let id_type = &self.id_field.ty;
      let id_ident = &self.id_field.ident;
      
      let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();
      let crate_name = crate::types::crate_name();

      quote! {
          impl #impl_generics ::#crate_name::traits::Model for #name #ty_generics #where_clause {
              type Id = #id_type;

              fn get_id(&self) -> Option<Self::Id> {
                  Some(self.#id_ident.clone())
              }
          }
      }
  }
}

impl ToTokens for DeriveModel {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let expanded = self.expand();

        tokens.extend(expanded);
    }
}
