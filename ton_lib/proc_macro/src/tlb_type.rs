use proc_macro::TokenStream;
use syn::__private::quote::__private::ext::RepToTokensExt;
use syn::__private::ToTokens;
use syn::DeriveInput;
use syn::parse::{Parse, ParseStream};

pub(crate) fn tlb_type_derive_impl(mut input: DeriveInput) -> TokenStream {

    // Extract a description, modifying `input.attrs` to remove the matched attributes.
    let tlb_prefix = match deluxe::extract_attributes(&mut input) {
        Ok(desc) => desc,
        Err(e) => return e.into_compile_error().into()
    };
    
    let args = TLBPrefix::new(&input.attrs);
    println!("{:#?}", &input.attrs);
    println!("{:#?}", &input.data);
    println!("{:#?}", &input.generics);
    // Parse attribute arguments

    TokenStream::new()

    // let expanded = quote! {
    //     impl #struct_name {
    //         pub const OPCODE: &'static str = #opcode;
    //         pub const DESCRIPTION: &'static str = #descr;
    //     }
    //     
    //     #input
    // };
    // 
    // TokenStream::from(expanded)
}

#[derive(deluxe::ExtractAttributes)]
#[deluxe(attributes(tlb_prefix))] // Match only `my_desc` attributes
struct TLBPrefix {
    value: Option<u128>,
    bits_len: Option<u32>,
}

impl TLBPrefix {
    fn new(attrs: &[syn::Attribute]) -> TLBPrefix {
        let mut opcode = None;
        let mut descr = None;



        for attr in attrs {
            if attr.meta.path().segments.iter().find(|x| x.ident == "tlb_prefix").is_none() {
                continue;
            }
            let mut token_stream = attr.meta.to_token_stream();
            let mut prefix = None;
            loop {
                let token = match token_stream.next() {
                    Some(token) => token,
                    None => break,
                };
                if let Some(token) = token.to_string().strip_prefix("value") {
                    token_stream.next();
                    let val = match token_stream.next() {
                        Some(token) => token,
                        None => break,
                    };
                    
                    prefix = Some(token.to_string());
                }
                let ident =
            }
            while let Some(token) = token_stream.next() {}
            for token in attr.meta.to_token_stream() {
                println!("{:#?}", token);
            }
            if attr.path.is_ident("opcode") {
                if let Ok(value) = attr.parse_args::<String>() {
                    opcode = Some(value);
                }
            } else if attr.path.is_ident("descr") {
                if let Ok(value) = attr.parse_args::<String>() {
                    descr = Some(value);
                }
            }
        }

        TLBPrefix { prefix: opcode, schema: descr }
    }
}

impl Parse for TLBPrefix {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        println!("{:?}", input);
        Ok(TLBPrefix {
            prefix: None,
            schema: None,
        })
    }
}