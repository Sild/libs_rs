use crate::{TLBFieldAttrs, TLBHeaderAttrs};
use proc_macro2::TokenStream;
use quote::quote;
use std::process::exit;
use syn::{DataStruct, Fields};

struct FieldInfo {
    ident: syn::Ident,
    attrs: TLBFieldAttrs,
}

pub(crate) fn tlb_derive_struct(header_attrs: &TLBHeaderAttrs, data: &mut DataStruct) -> (TokenStream, TokenStream) {
    let fields = match &mut data.fields {
        Fields::Named(fields) => &mut fields.named, // For struct { field1: T, field2: T }
        Fields::Unnamed(fields) => &mut fields.unnamed, // For tuple struct (T, T)
        Fields::Unit => panic!("MyDerive only supports structs"),
    };

    let fields_info = fields
        .iter_mut()
        .map(|f| {
            let ident = &f.ident;

            let field_attrs: TLBFieldAttrs = match deluxe::extract_attributes(&mut f.attrs) {
                Ok(desc) => desc,
                Err(_err) => exit(777),
            };
            FieldInfo {
                ident: ident.clone().unwrap(),
                attrs: field_attrs,
            }
        })
        .collect::<Vec<_>>();

    let mut read_def_str = fields_info
        .iter()
        .map(|f| {
            let ident = &f.ident;
            if let Some(bits_len) = f.attrs.bits_len {
                quote!(
                    let ident_tmp: ConstLen<_, #bits_len> = TLBType::read(parser)?;
                    let #ident = ident_tmp.0;
                )
            } else {
                quote!(let #ident = TLBType::read(parser)?;)
            }
        })
        .collect::<Vec<_>>();

    if *header_attrs.ensure_empty.as_ref().unwrap_or(&false) {
        read_def_str.push(quote!(parser.ensure_empty()?;));
    }

    let init_obj_str = fields_info
        .iter()
        .map(|f| {
            let ident = &f.ident;
            quote!(#ident,)
        })
        .collect::<Vec<_>>();

    let write_def_str = fields_info
        .iter()
        .map(|f| {
            let ident = &f.ident;
            if let Some(bits_len) = f.attrs.bits_len {
                quote!(
                    let tmp_ident = ConstLen::<_, #bits_len>(&self.#ident);
                    tmp_ident.write(dst)?;
                )
            } else {
                quote!(self.#ident.write(dst)?;)
            }
        })
        .collect::<Vec<_>>();

    let read_impl_token = quote::quote! {
        #(#read_def_str)*
        Ok(Self {
            #(#init_obj_str)*
        })
    };

    let write_impl_token = quote::quote! {
        #(#write_def_str)*
        Ok(())
    };
    (read_impl_token, write_impl_token)
}
