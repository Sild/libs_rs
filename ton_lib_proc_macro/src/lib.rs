mod tlb_derive_enum;
mod tlb_derive_struct;

use crate::tlb_derive_enum::tlb_derive_enum;
use crate::tlb_derive_struct::tlb_derive_struct;
use proc_macro::TokenStream;
use syn::Data;

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(tlb_derive))]
struct TLBHeaderAttrs {
    prefix: Option<u128>,       // use 0 as default
    bits_len: Option<u32>,      // use 0 as default
    ensure_empty: Option<bool>, // use false as default
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(tlb_derive))]
struct TLBFieldAttrs {
    bits_len: Option<u32>,
}

/// Automatic `TLBType` implementation
// #[derive(ton_lib_proc_macro::TLBDerive)]
// #[tlb_derive(prefix="0x12345678", bits_len=32, ensure_empty=true)]
// struct MyStruct {}
#[proc_macro_derive(TLBDerive, attributes(tlb_derive))]
pub fn tlb_derive(input: TokenStream) -> TokenStream {
    let mut input = syn::parse::<syn::DeriveInput>(input).unwrap();
    // Extract a description, modifying `input.attrs` to remove the matched attributes.
    let header_attrs: TLBHeaderAttrs = match deluxe::extract_attributes(&mut input) {
        Ok(desc) => desc,
        Err(e) => return e.into_compile_error().into(),
    };

    let ident = &input.ident;

    let (read_def_tokens, write_def_tokens) = match &mut input.data {
        Data::Struct(data) => tlb_derive_struct(&header_attrs, data),
        Data::Enum(data) => tlb_derive_enum(ident, data),
        _ => panic!("TLBDerive only supports structs and enums"),
    };

    let prefix_val = header_attrs.prefix.unwrap_or(0);
    let prefix_bits_len = header_attrs.bits_len.unwrap_or(0);

    quote::quote! {
        impl TLBType for #ident {
            const PREFIX: TLBPrefix = TLBPrefix::new(#prefix_val, #prefix_bits_len);

            fn read_definition(parser: &mut CellParser) -> Result<Self, TonLibError> {
                #read_def_tokens
            }

            fn write_definition(&self, dst: &mut CellBuilder) -> Result<(), TonLibError> {
                #write_def_tokens
            }
        }
    }
    .into()
}
