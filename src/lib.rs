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

use proc_macro2::TokenStream;
use quote::quote;
use syn::{Token, Expr, ExprAssign};
use syn::parse::Parser;
use syn::punctuated::Punctuated;

use crate::base::Base;
use crate::field::Field;
use crate::location::OnOverflow;
use crate::template::Template;
use crate::r#type::{Type, Precision};

// TODO:
// * Put compile checks behind different target so compiler updates don't break building.
// * Determine if Precision is relevant for replacebits (compare to splitbits).
// * Determine if truncate or panic should be the default for combinebits and replacebits.
// * splitbits_named_into isn't into-ing.
// * Add comments that show example macro expansion fragments.
// * Split tests into multiple files.
// After 0.1.0:
// * Create abstract syntax trees instead of quoting prematurely.
// * Extract argument parsing.
// * Ensure overflow behavior usability in const contexts.
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
    let (value, template, min_size) =
        parse_splitbits_input(input.into(), base, precision);
    let fields = template.extract_fields(&value, min_size);

    let struct_name = template.to_struct_name();
    let names: Vec<_> = fields.iter().map(|field| field.name().to_ident()).collect();
    let types: Vec<_> = fields.iter().map(|field| field.bit_width().to_token_stream()).collect();
    let values: Vec<TokenStream> = fields.iter().map(Field::to_token_stream).collect();
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
    let (value, template, min_size) =
        parse_splitbits_input(input.into(), base, precision);
    let fields = template.extract_fields(&value, min_size);
    let values: Vec<TokenStream> = fields.iter().map(Field::to_token_stream).collect();

    if let [value] = &values[..] {
        // Single value
        quote! { #value }
    } else {
        // Tuple
        quote! { (#(#values,)*) }
    }.into()
}

fn splitbits_named_into_base(
    input: proc_macro::TokenStream,
    base: Base,
    precision: Precision,
) -> proc_macro::TokenStream {
    let (value, template, min_size) =
        parse_splitbits_input(input.into(), base, precision);
    let fields = template.extract_fields(&value, min_size);
    let values: Vec<TokenStream> = fields.iter().map(Field::to_token_stream).collect();

    if let [value] = &values[..] {
        // Single value
        quote! { #value.into() }
    } else {
        // Tuple
        quote! { (#((#values).into(),)*) }
    }.into()
}

fn combinebits_base(
    input: proc_macro::TokenStream,
    base: Base,
) -> proc_macro::TokenStream {
    let parts = Parser::parse2(Punctuated::<Expr, Token![,]>::parse_terminated, input.into())
        .expect("combinebits! argument list should be formatted sanely");
    let mut parts: Vec<_> = parts.into_iter().collect();
    assert!(!parts.is_empty(), "combinebits! must take at least one argument (the template).");

    let mut on_overflow = OnOverflow::Truncate;
    // If we've got more than one argument, the first one might be an overflow setting.
    if let [assignment, _, ..] = &parts[..] &&
       let Some((setting, value)) = parse_assignment(assignment) {
        assert_eq!(setting, "overflow",
            "Only the 'overflow' setting is supported, but found '{setting}'.");
        parts.remove(0);
        on_overflow = OnOverflow::parse(&value)
            .expect("Valid overflow setting value must be passed");
    }

    let template = Template::from_expr(&parts.pop().unwrap(), base, Precision::Ux);
    if parts.is_empty() {
        // No arguments passed, so take them from the variables preceeding the macro instead.
        template.combine_with_context(on_overflow).into()
    } else {
        template.combine_with_args(on_overflow, &parts[..]).into()
    }
}

fn split_then_combine_base(input: proc_macro::TokenStream, base: Base) -> proc_macro::TokenStream {
    const PRECISION: Precision = Precision::Standard;
    let parts = Parser::parse2(Punctuated::<Expr, Token![,]>::parse_terminated, input.into())
        .expect("splitbits_then_combine! argument list should be formatted sanely");
    let parts: Vec<Expr> = parts.into_iter().collect();
    assert!(parts.len() >= 3);
    assert!(parts.len() % 2 == 1);

    let mut fields = Vec::new();
    for i in 0..parts.len() / 2 {
        let value = parts[2 * i].clone();
        let template = Template::from_expr(&parts[2 * i + 1], base, PRECISION);
        fields = Field::merge(&fields, &template.extract_fields(&value, None));
    }

    let expr = &parts[parts.len() - 1];
    let target = Template::from_expr(expr, base, PRECISION);
    if target.has_placeholders() {
        let bad_template = Template::template_string(expr);
        panic!(
            "Target template ({bad_template}) must not have placeholders (periods) in it. \
            Use literals instead as appropriate.");
    }

    let result = target.substitute_fields(fields);
    result.into()
}

fn replacebits_base(
    input: proc_macro::TokenStream,
    base: Base,
    precision: Precision,
) -> proc_macro::TokenStream {
    let parts = Parser::parse2(Punctuated::<Expr, Token![,]>::parse_terminated, input.clone().into())
        .expect("replacebits! argument list should be formatted sanely");
    let mut parts: Vec<_> = parts.into_iter().collect();
    assert!(parts.len() > 1,
        "replacebits must take at least two arguments: \
        an input value then a template. Found:\n`{input}`");
    assert!(parts.len() <= 3,
        "replacebits must take at most three arguments: \
        an overflow setting, then an input value, then a template. Found:\n`{input}`");

    let mut on_overflow = OnOverflow::Panic;
    if parts.len() == 3 {
        let (setting, value) = parse_assignment(&parts[0])
            .expect("the first argument to be an 'overflow' setting since three arguments were supplied");
        assert_eq!(setting, "overflow", "Only 'overflow' is allowed as a setting.");
        on_overflow = OnOverflow::parse(&value)
            .unwrap_or_else(|err_string| panic!("Invalid type for setting 'overflow'. {err_string}"));

        parts.remove(0);
    }

    let value = parts[0].clone();
    let template = Template::from_expr(&parts[1], base, precision);
    let result = template.replace(on_overflow, &value);
    result.into()
}

fn parse_splitbits_input(
    item: TokenStream,
    base: Base,
    precision: Precision,
) -> (Expr, Template, Option<Type>) {
    let parts = Parser::parse2(Punctuated::<Expr, Token![,]>::parse_terminated, item.clone())
        .expect("splitbits! argument list should be formatted sanely");
    let mut parts: Vec<_> = parts.into_iter().collect();
    assert!(parts.len() > 1,
        "splitbits must take at least two arguments: \
        an input value then a template. Found:\n`{item}`");
    assert!(parts.len() <= 3,
        "splitbits must take at most three arguments: \
        a min type, then an input value, then a template. Found:\n`{item}`");

    let mut min_size = None;
    if parts.len() == 3 {
        let (setting, value) = parse_assignment(&parts[0])
            .expect("the first argument to be a 'min' setting since three arguments were supplied");
        assert_eq!(setting, "min", "Only 'min' is allowed as a setting.");
        let size = Type::parse(value)
            .unwrap_or_else(|err_string| panic!("Invalid type for setting 'min'. {err_string}"));
        assert!(precision != Precision::Standard || size.is_standard(), "Type '{size}' is only supported in _ux macros.");
        min_size = Some(size);

        parts.remove(0);
    }

    let template_string = Template::template_string(&parts[1]);
    for c in template_string.chars() {
        assert!(!c.is_numeric() && !c.is_ascii_uppercase(),
            "Literals not allowed in this context, but found '{c}' in '{template_string}'.");
    }

    let value = parts[0].clone();
    let template = Template::from_expr(&parts[1], base, precision);
    (value, template, min_size)
}

fn parse_assignment(expr: &Expr) -> Option<(String, String)> {
    if let Expr::Assign(ExprAssign { left, right, ..}) = expr {
        let left = expr_to_ident(left)
            .expect("Setting name must be entirely alphabetical characters");
        let right = expr_to_ident(right)
            .expect("Setting value must be entirely alphabetical characters");
        Some((left, right))
    } else {
        None
    }
}

fn expr_to_ident(expr: &Expr) -> Result<String, String> {
    if let Expr::Path(path) = expr {
        path.path.get_ident()
            .ok_or_else(|| format!("Can't convert expr path to a setting component. Expr path: {path:?}"))
            .map(|ident| ident.to_string())
    } else {
        Err(format!("Can't convert expr to a setting component. Expr: {expr:?}"))
    }
}
