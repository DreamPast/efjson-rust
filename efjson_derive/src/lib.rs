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
  let ident = &input.ident;

  let span = proc_macro2::Span::mixed_site();
  let mut stream = TokenStream::new();

  let fields = &data.fields;
  let field_cnt = fields.len();
  if field_cnt == 0 {
    panic!("the struct must contain at least a member");
  }

  let union_ident = format_ident!("__EfjsonUnion_{}", input.ident, span = span);
  let struct_ident = format_ident!("__EfjsonStruct_{}", input.ident, span = span);

  stream.extend({
    let mut content = quote_spanned! {span=>};
    for field in fields {
      let ident = &field.ident;
      let typ = &field.ty;
      content.extend(quote_spanned! {span=>
        #ident: ::std::mem::ManuallyDrop<
          <#typ as ::efjson::deserialize::DefaultDeserializable<#typ>>::DefaultDeserializer
        >
      });
      content.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
    }

    quote_spanned! {span=>
      #[allow(non_camel_case_types)] union #union_ident { #content }
      #[allow(non_camel_case_types)]
      struct #struct_ident {
        target: ::std::mem::MaybeUninit<#ident>,
        subreceiver: ::std::mem::MaybeUninit<#union_ident>,
        index: usize,
        flag: [bool; #field_cnt],
      }
    }
  });

  stream.extend({
    let mut start_content = quote_spanned! {span=>};
    for (idx, field) in fields.iter().enumerate() {
      let ident = &field.ident;
      let typ = &field.ty;
      start_content.extend(quote_spanned! {span=>
        ::std::stringify!(#ident) => {
          if self.flag[#idx] {
            Err(format!("repeated key: {}", key).into())
          } else {
            unsafe {
              ::std::ptr::write(&mut (*self.subreceiver.as_mut_ptr()).#ident
                as &mut <#typ as ::efjson::deserialize::DefaultDeserializable<#typ>>::DefaultDeserializer,
                <#typ as ::efjson::deserialize::DefaultDeserializable<#typ>>::default_deserializer())
            };
            self.index = #idx;
            Ok(())
          }
        }
      });
      start_content.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
    }

    let mut feed_content = quote_spanned! {span=>};
    for (idx, field) in fields.iter().enumerate() {
      let ident = &field.ident;
      let typ = &field.ty;
      feed_content.extend(quote_spanned! {span=>
        #idx => match ::efjson::deserialize::Deserializer::feed_token(
          unsafe {
            &mut self.subreceiver.assume_init_mut().#ident
              as &mut <#typ as ::efjson::deserialize::DefaultDeserializable<#typ>>::DefaultDeserializer
          },
          token,
        )? {
          ::efjson::deserialize::DeserResult::Complete(r) => {
            unsafe { ::std::ptr::write(&mut (*self.target.as_mut_ptr()).#ident, r) };
            self.flag[#idx] = true;
            unsafe { ::std::mem::ManuallyDrop::drop(&mut self.subreceiver.assume_init_mut().#ident) };
            self.index = usize::max_value();
            ::efjson::deserialize::DeserResult::Complete(())
          }
          ::efjson::deserialize::DeserResult::CompleteWithRollback(r) => {
            unsafe { ::std::ptr::write(&mut (*self.target.as_mut_ptr()).#ident, r) };
            self.flag[#idx] = true;
            unsafe { ::std::mem::ManuallyDrop::drop(&mut self.subreceiver.assume_init_mut().#ident) };
            self.index = usize::max_value();
            ::efjson::deserialize::DeserResult::CompleteWithRollback(())
          }
          ::efjson::deserialize::DeserResult::Continue => ::efjson::deserialize::DeserResult::Continue
        }
      });
      feed_content.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
    }

    let mut end_content = quote_spanned! {span=>};
    for (idx, field) in fields.iter().enumerate() {
      let ident = &field.ident;
      end_content.extend(quote_spanned! {span=>
        if !self.flag[#idx] { list.push(::std::stringify!(#ident)); }
      });
    }

    quote_spanned! {span=>
      impl ::efjson::deserialize::StructHelperReceiverTrait<#ident> for #struct_ident {
        fn start_value(&mut self, key: &str) -> ::std::result::Result<(), ::efjson::deserialize::DeserError> {
          match key {
            #start_content
            _ => ::std::result::Result::Err(::std::format!("unxepcted key: {}", key).into()),
          }
        }
        fn feed_value(
          &mut self,
          token: ::efjson::stream_parser::Token,
        ) -> ::std::result::Result<::efjson::deserialize::DeserResult<()>, ::efjson::deserialize::DeserError> {
          Ok(match self.index {
            #feed_content
            _ => unsafe { ::std::hint::unreachable_unchecked() },
          })
        }
        fn end(&mut self) -> ::std::result::Result<#ident, ::efjson::deserialize::DeserError> {
          if self.flag.iter().all(|f| *f) {
            self.flag.fill(false);
            Ok(unsafe { self.target.assume_init_read() })
          } else {
            let mut list: Vec<&str> = Vec::new();
            #end_content
            Err(::std::format!("missing required fileds: {}", list.join(", ")).into())
          }
        }
      }
    }
  });

  stream.extend({
    let mut deser_drop_content = quote_spanned! {span=>};
    for (idx, field) in fields.iter().enumerate() {
      let ident = &field.ident;
      deser_drop_content.extend(quote_spanned! {span=>
        #idx => unsafe { ::std::mem::ManuallyDrop::drop(&mut self.subreceiver.assume_init_mut().#ident) }
      });
      deser_drop_content.append(proc_macro2::Punct::new(',', proc_macro2::Spacing::Alone));
    }

    let mut val_drop_content = quote_spanned! {span=>};
    for (idx, field) in fields.iter().enumerate() {
      let ident = &field.ident;
      val_drop_content.extend(quote_spanned! {span=>
        if self.flag[#idx] {
          unsafe { ::std::ptr::drop_in_place(&mut (*self.target.as_mut_ptr()).#ident) };
        }
      });
    }

    quote_spanned! {span=>
      impl ::std::ops::Drop for #struct_ident {
        fn drop(&mut self) {
          match self.index {
            #deser_drop_content
            _ => {}
          }
          #val_drop_content
        }
      }
    }
  });

  stream.extend({
    quote_spanned! {span=>
      impl ::efjson::deserialize::DefaultDeserializable<#ident> for #ident {
        type DefaultDeserializer
          = ::efjson::deserialize::StructHelperReceiverDeserializer<#ident, #struct_ident>;
        fn default_deserializer() -> Self::DefaultDeserializer {
          ::efjson::deserialize::create_struct_helper_deserializer(#struct_ident {
            flag: ::std::default::Default::default(),
            index: usize::max_value(),
            target: ::std::mem::MaybeUninit::uninit(),
            subreceiver: ::std::mem::MaybeUninit::uninit(),
          })
        }
      }
    }
  });

  // println!("{:}", stream.to_string());
  stream.into()
}
