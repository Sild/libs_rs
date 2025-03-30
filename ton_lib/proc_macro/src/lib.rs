mod tlb_type;

use proc_macro::TokenStream;
use crate::tlb_type::tlb_type_derive_impl;



#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(tlb_prefix))] // Match only `tlb_prefix` attributes
struct TLBPrefix {
    value: Option<u128>,
    bits_len: Option<u32>,
}

/// Implements `TLBType` for the type.
/// Usage:
/// #[derive(TLBType)]
/// #[tlb_prefix(value="0x12345678", bit_len=32)]
/// struct MyStruct {}
#[proc_macro_derive(TLBType, attributes(tlb_prefix))]
pub fn derive_tlb_prefix(input: TokenStream) -> TokenStream {
    let mut input = syn::parse::<syn::DeriveInput>(input).unwrap();
    let prefix = match deluxe::extract_attributes(&mut input) {
        Ok(desc) => desc,
        Err(e) => return e.into_compile_error().into(),
    };
    
    tlb_type_derive_impl(input)
}