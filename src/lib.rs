#![forbid(unsafe_code)]

extern crate proc_macro;

mod base;
mod character;
mod field;
mod location;
mod name;
mod segment;
mod template;
mod r#type;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Token, Expr};
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use crate::base::Base;
use crate::field::Field;
use crate::template::Template;
use crate::r#type::Precision;

// TODO:
// * Enable specifying overflow behavior in combinebits.
// * Add splitbits_capture.
// * Add build-your-own splitbits with other Bases.
// * Allow const variable templates.
// * Allow passing minimum variable size.
// * Allow non-const variable templates (as a separate macro?).
// * Better error messages.
// ** Calling splitbits with hex values should give a tailored message.
// * Remove itertools dependency.
// * Allow non-standard template lengths.
// * Tests that confirm non-compilation cases.
// * splitbits_named_into isn't into-ing.
// * All replacebits variants.
#[proc_macro]
pub fn splitbits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Binary, Precision::Standard)
}

#[proc_macro]
pub fn splitbits_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Binary, Precision::Ux)
}

#[proc_macro]
pub fn splithex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Hexadecimal, Precision::Standard)
}

#[proc_macro]
pub fn splithex_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Hexadecimal, Precision::Ux)
}

#[proc_macro]
pub fn splitbits_named(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_base(input, Base::Binary, Precision::Standard)
}

#[proc_macro]
pub fn splitbits_named_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_base(input, Base::Binary, Precision::Ux)
}

#[proc_macro]
pub fn splithex_named(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_base(input, Base::Hexadecimal, Precision::Standard)
}

#[proc_macro]
pub fn splithex_named_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_base(input, Base::Hexadecimal, Precision::Ux)
}

#[proc_macro]
pub fn splitbits_named_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_into_base(input, Base::Binary, Precision::Standard)
}

#[proc_macro]
pub fn splitbits_named_into_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_into_base(input, Base::Binary, Precision::Ux)
}

#[proc_macro]
pub fn splithex_named_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_into_base(input, Base::Hexadecimal, Precision::Standard)
}

#[proc_macro]
pub fn splithex_named_into_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_into_base(input, Base::Hexadecimal, Precision::Ux)
}

#[proc_macro]
pub fn combinebits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    combinebits_base(input, Base::Binary)
}

#[proc_macro]
pub fn combinehex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    combinebits_base(input, Base::Hexadecimal)
}

#[proc_macro]
pub fn splitbits_then_combine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    split_then_combine_base(input, Base::Binary)
}

#[proc_macro]
pub fn splithex_then_combine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    split_then_combine_base(input, Base::Hexadecimal)
}

#[proc_macro]
pub fn replacebits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    /*
     * self.address &= 0b1111_1111_0000_0000;
     * self.address |= u16::from(value);
    */
    // replacebits!(self.address, ".... .... vvvv vvvv");

    let (value, template) = parse_input(input.into(), Base::Binary, Precision::Standard);
    let result = template.replace(&value);
    println!("{result}");
    result.into()
}

fn splitbits_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into(), base, precision);
    let fields = template.extract_fields(&value);

    let struct_name = template.to_struct_name();
    let names: Vec<_> = fields.iter().map(|field| field.name().to_ident()).collect();
    let types: Vec<_> = fields.iter().map(|field| field.t().to_token_stream()).collect();
    let values: Vec<TokenStream> = fields.iter().map(|field| field.to_token_stream()).collect();
    let result = quote! {
        {
            struct #struct_name {
                #(#names: #types,)*
            }

            #struct_name {
                #(#names: #values,)*
            }
        }
    };

    result.into()
}

fn splitbits_named_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into(), base, precision);
    let fields = template.extract_fields(&value);
    let values: Vec<TokenStream> = fields.iter().map(|field| field.to_token_stream()).collect();

    let result = match &values[..] {
        // Single value
        [value] => quote! { #value },
        // Tuple
        _ => quote! { (#(#values,)*) },
    };

    result.into()
}

fn splitbits_named_into_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template) = parse_input(input.into(), base, precision);
    let fields = template.extract_fields(&value);
    let values: Vec<TokenStream> = fields.iter().map(|field| field.to_token_stream()).collect();

    let result = match &values[..] {
        // Single value
        [value] => quote! { #value.into() },
        // Tuple
        _ => quote! { (#((#values).into(),)*) },
    };

    result.into()
}

fn combinebits_base(input: proc_macro::TokenStream, base: Base) -> proc_macro::TokenStream {
    let parts = Parser::parse2(
        Punctuated::<Expr, Token![,]>::parse_terminated,
        input.clone().into(),
    ).unwrap();
    let parts: Vec<Expr> = parts.into_iter().collect();
    if parts.len() == 1 {
        let template = Template::from_expr(&parts[0], base, Precision::Ux);
        template.combine_variables().into()
    } else {
        let template = Template::from_expr(&parts[parts.len() - 1], base, Precision::Ux);
        // Everything except the last argument is an input variable.
        template.combine_with(&parts[0..parts.len() - 1]).into()
    }
}

fn split_then_combine_base(input: proc_macro::TokenStream, base: Base) -> proc_macro::TokenStream {
    const PRECISION: Precision = Precision::Standard;

    let parts = Parser::parse2(
        Punctuated::<Expr, Token![,]>::parse_terminated,
        input.clone().into(),
    ).unwrap();
    let parts: Vec<Expr> = parts.into_iter().collect();
    assert!(parts.len() >= 3);
    assert!(parts.len() % 2 == 1);

    let mut fields = Vec::new();
    for i in 0..parts.len() / 2 {
        let value = parts[2 * i].clone();
        let template = Template::from_expr(&parts[2 * i + 1], base, PRECISION);
        fields = Field::merge(fields, template.extract_fields(&value));
    }

    let expr = &parts[parts.len() - 1];
    let target = Template::from_expr(expr, base, PRECISION);
    if target.has_placeholders() {
        let bad_template = Template::template_string(expr);
        panic!(
            "Target template ({bad_template}) must not have placeholders (periods) in it. Use literals instead as appropriate.",
        );
    }

    let result = target.substitute_fields(fields);
    result.into()
}

fn parse_input(item: TokenStream, base: Base, precision: Precision) -> (Expr, Template) {
    let parts = Parser::parse2(
        Punctuated::<Expr, Token![,]>::parse_terminated,
        item.clone().into(),
    ).unwrap();
    let parts: Vec<Expr> = parts.into_iter().collect();
    assert_eq!(parts.len(), 2);

    let value = parts[0].clone();
    let template = Template::from_expr(&parts[1], base, precision);
    (value, template)
}
