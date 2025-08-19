use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Ident, Variant
};
use std::option::Option::Some;
use std::option::Option::None;

#[proc_macro_derive(Schema, attributes(schema))]
pub fn derive(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = input.ident;

    let varaints_to_names = match &input.data {
      Data::Enum(ref data) => {
        let varaints_to_names = data.variants.iter().map(|v| {
          let v_name = &v.ident;
          let v_name_str = v_name.to_string();
          let base_selector = quote_spanned! {
            v.span()=>
              #name::#v_name
          };
          if &v_name_str == "SchemaType" {
            return quote_spanned! {
              v.span()=>
                #base_selector(_) => vec![],
            };
          }
          let base_variant_type = quote_spanned! {
            v.span()=>
              #name::SchemaType(#v_name_str.to_string()),
          };

          let attrs: Vec<_> = v.attrs.iter().map(|attr| {
            use syn::Meta::*;

            match &attr.meta {
              List(meta) => {
                let kv = meta.tokens.to_string();
                let (attr_name, value) = kv.split_once("=").unwrap();
                (attr_name.trim().to_string(), value.trim().to_string(), meta)
              },
              _ => unimplemented!()
            }

          }).collect();
          additional_types.insert(0, quote_spanned! {
            v.span()=>
              #name::SchemaType(#v_name_str.to_string()),
          if let Some((attr_name, value, meta)) = attrs.first() {
            match attr_name.as_str() {
              "additional_types" => {
                let mut types: Vec<_> = value.split(",").into_iter().map(|v| {
                  let t_name = v.trim();
                  quote_spanned! {
                    v.span()=>
                      #name::SchemaType(#t_name.to_string()),
                  }
                }).collect();
                types.insert(0, base_variant_type);
                let base_selector = base_selector_ignore_fields(v, base_selector);
                return quote_spanned! {
                  meta.span()=>
                    #base_selector => vec![#(#types)*],
                }
              }
              "custom" => {
                let value = Ident::new(value, Span::call_site());
                let args: Vec<_> = v.fields.iter().map(|field| {
                  let name = field.ident.clone();
                  quote_spanned! {
                    field.span() =>
                      &#name
                  }
                }).collect();
          let base_selector = base_selector_with_fields(v, base_selector);
                return quote_spanned! {
                  meta.span() =>
                    #base_selector => #value(#(#args,)*),
                }
              }
              _ => unimplemented!("attribute '{}' not supported", attr_name)
            }
          }

          let base_selector = base_selector_ignore_fields(v, base_selector);
          quote_spanned! {
            v.span()=>
              #base_selector => vec![#base_variant_type],
          }
        });
        quote! {
          #(#varaints_to_names)*
        }
      }
      Data::Struct(_) | Data::Union(_) => unimplemented!(),
    };


    let expanded = quote! {
      use gravitydb::schema::NestableProperty;

      impl NestableProperty for #name {
        fn nested(&self) -> Vec<Self> {
          match self {
            #varaints_to_names
          }
        }
      }
    };

    TokenStream::from(expanded).into()
}

fn base_selector_ignore_fields(v: &Variant, base: TokenStream) -> TokenStream {
  match &v.fields {
    Fields::Named(f) => {
      let ignored_fields = f.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! {
          f.span()=>
            #name: _,
        }
      });
      quote_spanned! {
        v.span()=>
          #base{#(#ignored_fields)*}
      }
    },
    Fields::Unnamed(f) => {
      let ignored_fields = f.unnamed.iter().map(|f| {
        quote_spanned! {
          f.span()=>
            _,
        }
      });
      quote_spanned! {
        v.span()=>
          #base(#(#ignored_fields)*)
      }
    }
    Fields::Unit => base,
  }
}

fn base_selector_with_fields(v: &Variant, base: TokenStream) -> TokenStream {
  match &v.fields {
    Fields::Named(f) => {
      let fields = f.named.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! {
          f.span()=>
            #name,
        }
      });
      quote_spanned! {
        v.span()=>
          #base{#(#fields)*}
      }
    },
    Fields::Unnamed(f) => {
      let fields = f.unnamed.iter().map(|f| {
        let name = &f.ident;
        quote_spanned! {
          f.span()=>
            #name,
        }
      });
      quote_spanned! {
        v.span()=>
          #base(#(#fields)*)
      }
    }
    Fields::Unit => base,
  }
}
