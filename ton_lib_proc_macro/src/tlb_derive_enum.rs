use deluxe::____private::Ident;
use proc_macro2::TokenStream;
use quote::quote;
use syn::{DataEnum, Fields};

pub(crate) fn tlb_derive_enum(ident: &Ident, data: &mut DataEnum) -> (TokenStream, TokenStream) {
    let variant_readers = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        // Expect single unnamed field (like `Std(...)`)
        let Fields::Unnamed(fields) = &variant.fields else {
            panic!("TryFromParser only supports tuple-like enums");
        };
        if fields.unnamed.len() != 1 {
            panic!("Each enum variant must have exactly one unnamed field");
        }
        let field_type = &fields.unnamed.first().unwrap().ty;
        quote! {
                match #field_type::read(parser) {
                    Ok(res) => return Ok(#ident::#variant_name(res)),
                    Err(TonLibError::TLBWrongPrefix { .. }) => {},
                    Err(err) => return Err(err),
                };
        }
    });

    let variant_writers = data.variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        // Expect single unnamed field (like `Std(...)`)
        let Fields::Unnamed(fields) = &variant.fields else {
            panic!("TryFromParser only supports tuple-like enums");
        };

        if fields.unnamed.len() != 1 {
            panic!("Each enum variant must have exactly one unnamed field");
        }
        quote! {
            Self::#variant_name(ref value) => value.write(dst)?,
        }
    });

    let read_impl = quote! {
        #(#variant_readers)*
        Err(TonLibError::TLBEnumOutOfOptions)
    };

    let write_impl = quote! {
        match self {
            #(#variant_writers)*
        }
        Ok(())
    };
    (read_impl, write_impl)
}
