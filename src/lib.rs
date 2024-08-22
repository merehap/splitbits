#![forbid(unsafe_code)]
#![feature(let_chains)]

extern crate proc_macro;

mod base;
mod character;
mod field;
mod location;
mod name;
mod segment;
mod template;
mod r#type;

use std::collections::VecDeque;

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Token, Expr, ExprAssign};
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use crate::base::Base;
use crate::field::Field;
use crate::template::{Template, OnOverflow};
use crate::r#type::{Type, Precision};

// TODO:
// * Ensure overflow behavior usability in const contexts.
// * Allow use of u1s instead of bools.
// * Better error messages.
// * Tests that confirm non-compilation cases.
// * splitbits_named_into isn't into-ing.
// * Support splitting inputs across multiple output segments.
// After 0.1.0:
// * Add base 8, base 32, and base 64.
// ** Add build-your-own splitbits with other Bases.
// * Enable splitbits to fail if literal pattern not matched
// * Allow const variable templates.
// * Allow non-const variable templates (as a separate macro).
// * Allow non-standard template lengths.
// * Add splitbits_capture.
// * Add file-level config for overflow and min.

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
    replacebits_base(input, Base::Binary, Precision::Standard)
}

#[proc_macro]
pub fn replacehex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    replacebits_base(input, Base::Hexadecimal, Precision::Standard)
}

#[proc_macro]
pub fn replacebits_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    replacebits_base(input, Base::Binary, Precision::Ux)
}

#[proc_macro]
pub fn replacehex_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    replacebits_base(input, Base::Hexadecimal, Precision::Ux)
}

fn splitbits_base(
    input: proc_macro::TokenStream,
    base: Base,
    precision: Precision,
) -> proc_macro::TokenStream {
    let (value, template, min_size) = parse_input(input.into(), base, precision, LiteralsAllowed::No);
    let fields = template.extract_fields(&value, min_size);

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

fn splitbits_named_base(
    input: proc_macro::TokenStream,
    base: Base,
    precision: Precision,
) -> proc_macro::TokenStream {
    let (value, template, min_size) = parse_input(input.into(), base, precision, LiteralsAllowed::No);
    let fields = template.extract_fields(&value, min_size);
    let values: Vec<TokenStream> = fields.iter().map(|field| field.to_token_stream()).collect();

    let result = match &values[..] {
        // Single value
        [value] => quote! { #value },
        // Tuple
        _ => quote! { (#(#values,)*) },
    };

    result.into()
}

fn splitbits_named_into_base(
    input: proc_macro::TokenStream,
    base: Base,
    precision: Precision,
) -> proc_macro::TokenStream {
    let (value, template, min_size) = parse_input(input.into(), base, precision, LiteralsAllowed::No);
    let fields = template.extract_fields(&value, min_size);
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
    let mut parts: VecDeque<_> = parts.into_iter().collect();
    let on_overflow = match u8::try_from(parts.len()).unwrap() {
        0 => panic!("combinebits! must take at least one argument."),
        1 => OnOverflow::Wrap,
        2..=255 => {
            if let Some(Setting::Overflow(on_overflow)) = parse_assignment(&parts[0]) {
                parts.pop_front();
                on_overflow
            } else {
                OnOverflow::Wrap
            }
        }
    };

    if parts.len() == 1 {
        let template = Template::from_expr(&parts.pop_front().unwrap(), base, Precision::Ux);
        template.combine_variables(on_overflow).into()
    } else {
        let template = Template::from_expr(&parts.pop_back().unwrap(), base, Precision::Ux);
        // Everything except the last argument is an input variable.
        parts.make_contiguous();
        template.combine_with(parts.as_slices().0).into()
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
        fields = Field::merge(fields, template.extract_fields(&value, None));
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

fn replacebits_base(input: proc_macro::TokenStream, base: Base, precision: Precision) -> proc_macro::TokenStream {
    let (value, template, min_size) = parse_input(input.into(), base, precision, LiteralsAllowed::Yes);
    assert_eq!(min_size, None);
    let result = template.replace(&value);
    result.into()
}

fn parse_input(item: TokenStream, base: Base, precision: Precision, literals_allowed: LiteralsAllowed) -> (Expr, Template, Option<Type>) {
    let parts = Parser::parse2(
        Punctuated::<Expr, Token![,]>::parse_terminated,
        item.clone().into(),
    ).unwrap();
    let mut parts: VecDeque<_> = parts.into_iter().collect();
    assert!(parts.len() == 2 || parts.len() == 3);

    let mut min_size = None;
    if parts.len() == 3 {
        if let Some(Setting::MinFieldSize(size)) = parse_assignment(&parts[0]) {
            if precision == Precision::Standard && !size.is_standard() {
                panic!("Type '{size}' is only supported in _ux macros.");
            }
            min_size = Some(size);
        } else {
            panic!();
        }
        parts.pop_front();
    }

    if literals_allowed == LiteralsAllowed::No {
        let template_string = Template::template_string(&parts[1]);
        for c in template_string.chars() {
            assert!(!c.is_numeric() && !c.is_ascii_uppercase(),
                "Literals not allowed in this context, but found '{c}' in '{template_string}'.");
        }
    }

    let value = parts[0].clone();
    let template = Template::from_expr(&parts[1], base, precision);
    (value, template, min_size)
}

#[derive(PartialEq, Eq)]
enum LiteralsAllowed {
    No,
    Yes,
}

fn parse_assignment(expr: &Expr) -> Option<Setting> {
    if let Expr::Assign(ExprAssign { left, right, ..}) = expr {
        Some(Setting::parse(&left, &right).unwrap())
    } else {
        None
    }
}

enum Setting {
    Overflow(OnOverflow),
    MinFieldSize(Type),
}

impl Setting {
    fn parse(left: &Expr, right: &Expr) -> Result<Setting, String> {
        match Setting::expr_to_ident(left)?.as_ref() {
            "overflow" => {
                let overflow = match Setting::expr_to_ident(right)?.as_ref() {
                    "wrap" => OnOverflow::Wrap,
                    "panic" => OnOverflow::Panic,
                    "corrupt" => OnOverflow::Corrupt,
                    "saturate" => OnOverflow::Saturate,
                    overflow => return Err(
                        format!("'{overflow}' is an invalid overflow option. Options: 'wrap', 'panic', 'corrupt', 'saturate'.")),
                };
                Ok(Setting::Overflow(overflow))
            },
            "min" => {
                let mut min = Setting::expr_to_ident(right)?;
                let u = min.remove(0);
                assert_eq!(u, 'u');
                let min: u8 = min.parse().unwrap();
                Ok(Setting::MinFieldSize(Type::ux(min)))
            }
            name => return Err(format!("'{name}' is not a supported setting.")),
        }
    }

    fn expr_to_ident(expr: &Expr) -> Result<String, String> {
        if let Expr::Path(path) = expr {
            path.path.get_ident()
                .ok_or(format!("Can't convert expr path to a setting component. Expr path: {path:?}"))
                .map(|p| p.to_string())
        } else {
            Err(format!("Can't convert expr to a setting component. Expr: {expr:?}"))
        }
    }
}
