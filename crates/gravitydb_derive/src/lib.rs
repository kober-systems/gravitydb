use proc_macro2::{Span, TokenStream};
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields, Ident, MetaList, Variant
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

          let (additional_types, custom): (Vec<_>, Vec<_>) = get_attrs(&v)
            .into_iter().partition(|(attr_name, _, _)| match attr_name.as_str() {
              "additional_types" => true,
              "custom" => false,
              _ => unimplemented!("attribute '{}' not supported", attr_name),
            });
          let mut additional_types: Vec<_> = additional_types.iter().map(|(_attr_name, value, _meta)| {
            extract_additional_schema_types(&value, &name)
          }).flatten().collect();
          let custom: Vec<_> = custom.iter().map(|(_attr_name, value, meta)| {
              extract_custom_schema_type_function(&value, &v, meta)
          }).collect();

          let base_selector = if custom.len() > 0 {
            base_selector_with_fields(v, base_selector)
          } else {
            base_selector_ignore_fields(v, base_selector)
          };

          if let Some(custom_call) = custom.first() {
            return quote_spanned! {
              v.span() =>
                #base_selector => #custom_call,
            }
          }

          additional_types.insert(0, base_variant_type);
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

fn extract_additional_schema_types(value: &str, name: &Ident) -> Vec<TokenStream> {
  value.split(",").into_iter().map(|v| {
    let t_name = v.trim();
    quote_spanned! {
      v.span()=>
        #name::SchemaType(#t_name.to_string()),
    }
  }).collect()
}

fn extract_custom_schema_type_function(value: &str, variant: &Variant, meta: &MetaList) -> TokenStream {
  let value = Ident::new(value, Span::call_site());
  let args: Vec<_> = variant.fields.iter().map(|field| {
    let name = field.ident.clone();
    quote_spanned! {
      field.span() =>
        &#name
    }
  }).collect();
  quote_spanned! {
    meta.span() =>
      #value(#(#args,)*)
  }
}

fn get_attrs(v: &Variant) -> Vec<(String, String, &MetaList)> {
  v.attrs.iter().map(|attr| {
    use syn::Meta::*;

    match &attr.meta {
      List(meta) => {
        let kv = meta.tokens.to_string();
        let (attr_name, value) = kv.split_once("=").unwrap();
        (attr_name.trim().to_string(), value.trim().to_string(), meta)
      },
      _ => unimplemented!()
    }

  }).collect()
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
