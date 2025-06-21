extern crate proc_macro;
use proc_macro2::TokenStream;
use quote::{format_ident, quote_spanned, TokenStreamExt};
use syn::parse_macro_input;

#[proc_macro_derive(Deserializable, attributes())]
pub fn derive_answer_fn(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
  let input = parse_macro_input!(input as syn::DeriveInput);
  let data = match input.data {
    syn::Data::Union(_) => panic!("unions are not supported"),
    syn::Data::Enum(_) => panic!("TODO: enums are not supported"),
    syn::Data::Struct(item) => item,
  };
  let span = proc_macro2::Span::mixed_site();
  let mut stream = TokenStream::new();
  let fields = &data.fields;
  let ident = &input.ident;

  // define the value receiver enum, and impl `Deserializer` for it
  let enum_name = format_ident!("__EfjsonEnum_{}", input.ident, span = span);
  stream.extend({
    let mut enum_members = quote_spanned! {span=>};
    for field in fields {
      let ident = &field.ident;
      let typ = &field.ty;
      enum_members.extend(quote_spanned! {span=>
        #ident(<#typ as ::efjson::deserialize::DefaultDeserializable<#typ>>::DefaultDeserializer, *mut Option<#typ>)
      });
      enum_members.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
    }
    quote_spanned! {span=>
      #[allow(non_camel_case_types)]
      enum #enum_name { #enum_members }
    }
  });
  stream.extend({
    let mut receiver_content = quote_spanned! {span=>};
    for field in fields {
      let ident = &field.ident;
      receiver_content.extend(quote_spanned! {span=>
        Self::#ident(ref mut subr, ptr) => match subr.feed_token(token)? {
          efjson::deserialize::DeserResult::Complete(ret) => {
            let _ = unsafe { std::mem::replace(ptr.as_mut().unwrap_unchecked(), Some(ret)) };
            efjson::deserialize::DeserResult::Complete(())
          }
          efjson::deserialize::DeserResult::CompleteWithRollback(ret) => {
            let _ = unsafe { std::mem::replace(ptr.as_mut().unwrap_unchecked(), Some(ret)) };
            efjson::deserialize::DeserResult::CompleteWithRollback(())
          }
          efjson::deserialize::DeserResult::Continue => efjson::deserialize::DeserResult::Continue,
        }
      });
      receiver_content.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
    }

    quote_spanned! {span=>
      impl ::efjson::deserialize::Deserializer<()> for #enum_name {
        fn feed_token(
          &mut self, token: ::efjson::stream_parser::Token
        ) -> ::std::result::Result<::efjson::deserialize::DeserResult<()>, ::efjson::deserialize::DeserError>
        {
          Ok(match self { #receiver_content })
        }
      }
    }
  });

  // define the struct receiver, and impl `ObjectReceiverTrait` for it
  let struct_receiver_name = format_ident!("__EfjsonReceiver_{}", input.ident, span = span);
  stream.extend({
    let mut struct_receiver_fields = quote_spanned! {span=>};
    for field in fields {
      let ident = &field.ident;
      let ty = &field.ty;
      struct_receiver_fields.extend(quote_spanned! {span=>
        #ident: Option<#ty>
      });
      struct_receiver_fields.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
    }
    quote_spanned! {span=>
      #[derive(Default)]
      struct #struct_receiver_name {
        #struct_receiver_fields
      }
    }
  });
  stream.extend({
    let mut value_receivers = quote_spanned! {span=>};
    for field in fields {
      let ident = &field.ident;
      let typ = &field.ty;
      value_receivers.extend(quote_spanned! {span=>
        ::core::stringify!(#ident) => Ok(#enum_name::#ident(
          <#typ as ::efjson::deserialize::DefaultDeserializable<#typ>>::default_deserializer(),
          &mut self.#ident,
        ))
      });
      value_receivers.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
    }

    let mut return_content = quote_spanned! {span=>};
    for field in fields {
      let ident = &field.ident;
      return_content.extend(quote_spanned! {span=>
        #ident: self.#ident.take().ok_or_else(|| format!("missing field: {}", ::core::stringify!(#ident)))?
      });
      return_content.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
    }

    quote_spanned! {span=>
      impl
        ::efjson::deserialize::ObjectReceiverTrait<
          ::std::string::String,
          (),
          #ident,
          <::std::string::String as ::efjson::deserialize::DefaultDeserializable<
            ::std::string::String,
          >>::DefaultDeserializer,
          #enum_name,
        > for #struct_receiver_name
      {
        fn create_key(
          &mut self,
        ) -> ::std::result::Result<
          <::std::string::String as ::efjson::deserialize::DefaultDeserializable<
            ::std::string::String,
          >>::DefaultDeserializer,
          ::efjson::deserialize::DeserError,
        > {
          Ok(<::std::string::String as ::efjson::deserialize::DefaultDeserializable<
            ::std::string::String,
          >>::default_deserializer())
        }

        fn create_value(
          &mut self,
          key: &::std::string::String,
        ) -> ::std::result::Result<#enum_name, ::efjson::deserialize::DeserError> {
          match key as &::std::primitive::str {
            #value_receivers
            _ => Err(format!("unxepcted key: {}", key).into()),
          }
        }

        fn set(
          &mut self,
          _key: ::std::string::String,
          _value: (),
        ) -> ::std::result::Result<(), ::efjson::deserialize::DeserError> {
          Ok(())
        }

        fn end(&mut self) -> ::std::result::Result<#ident, ::efjson::deserialize::DeserError> {
          Ok(#ident {
            #return_content
          })
        }
      }
    }
  });

  // impl `DefaultDeserializable` for the struct
  stream.extend({
    let mut field_content = quote_spanned! {span=>};
    for field in fields {
      let ident = &field.ident;
      field_content.extend(quote_spanned! {span=>
        #ident: ::std::option::Option::None
      });
      field_content.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
    }

    quote_spanned! {span=>
      impl ::efjson::deserialize::DefaultDeserializable<#ident> for #ident {
        type DefaultDeserializer = ::efjson::deserialize::ObjectReceiverDeserializer<
          ::std::string::String,
          (),
          #ident,
          #struct_receiver_name,
          <::std::string::String as ::efjson::deserialize::DefaultDeserializable<
            ::std::string::String,
          >>::DefaultDeserializer,
          #enum_name,
        >;
        fn default_deserializer() -> Self::DefaultDeserializer {
          efjson::deserialize::create_object_deserializer(#struct_receiver_name { #field_content })
        }
      }
    }
  });

  // println!("{:}", stream.to_string());
  stream.into()
}
