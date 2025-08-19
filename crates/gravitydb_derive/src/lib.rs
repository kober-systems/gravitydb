use proc_macro2::TokenStream;
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
          let base_selector = get_base_selector_for_field(v, base_selector);
          let mut additional_types: Vec<_> = v.attrs.iter().map(|attr| {
            use syn::Meta::*;

            match &attr.meta {
              List(v) => {
                let kv = v.tokens.to_string();
                let (attr_name, value) = kv.split_once("=").unwrap();
                let attr_name = attr_name.trim();
                match attr_name {
                  "additional_types" => {
                    let types: Vec<_> = value.split(",").into_iter().map(|v| {
                      let t_name = v.trim();
                      quote_spanned! {
                        v.span()=>
                          #name::SchemaType(#t_name.to_string()),
                      }
                    }).collect();
                    quote_spanned! {
                      v.span()=>
                        #(#types)*
                    }
                  }
                  _ => unimplemented!("attribute '{}' not supported", attr_name)
                }
              },
              _ => unimplemented!()
            }

          }).collect();
          additional_types.insert(0, quote_spanned! {
            v.span()=>
              #name::SchemaType(#v_name_str.to_string()),
          });
          quote_spanned! {
            v.span()=>
              #base_selector => vec![#(#additional_types)*],
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

fn get_base_selector_for_field(v: &Variant, base: TokenStream) -> TokenStream {
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
