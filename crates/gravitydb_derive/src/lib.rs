use proc_macro2::TokenStream;
use quote::{quote, quote_spanned};
use syn::spanned::Spanned;
use syn::{
    parse_macro_input, Data, DeriveInput, Fields
};
use std::option::Option::Some;
use std::option::Option::None;

#[proc_macro_derive(Schema)]
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
          let base_selector = match &v.fields {
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
                  #base_selector{#(#ignored_fields)*}
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
                  #base_selector(#(#ignored_fields)*)
              }
            }
            Fields::Unit => base_selector,
          };
          quote_spanned! {
            v.span()=>
              #base_selector => vec![#name::SchemaType(#v_name_str.to_string())],
          }
        });
        quote! {
          #(#varaints_to_names)*
        }
      }
      Data::Struct(_) | Data::Union(_) => unimplemented!(),
    };


    let expanded = quote! {
      use gravitydb::schema::Property;

      impl<Error> Property<String, Error> for #name {
        fn nested(&self) -> Vec<Self> {
          match self {
            #varaints_to_names
          }
        }
      }
    };

    TokenStream::from(expanded).into()
}

