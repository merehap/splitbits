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
// * Detailed top-level comments.
// * Put compile checks behind different target so compiler updates don't break building.
// * splitbits_named_into isn't into-ing.
// * Split tests into multiple files.
// * Add missing variable test for splitbits.
// * Add wrong number of args test for splitbits.
// After 0.1.0:
// * Create abstract syntax trees instead of quoting prematurely.
// ** Add comments that show example macro expansion fragments.
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

/// Extract bit fields from an integer data type by matching against a template,
/// storing them as fields in a generated struct.
/// ```
/// use splitbits::splitbits;
///
/// let fields = splitbits!(0b11110000, "aaabbbbb");
/// // Single-letter field names, directly from the unique letters in the template above.
/// assert_eq!(fields.a, 0b111);
/// assert_eq!(fields.b, 0b10000);
/// ```
///
/// For hexadecimal templates (instead of binary), see [`splithex!`]
/// 
/// If single-letter variable names aren't enough, see [`splitbits_named!`]
///
/// The input variable can be any standard unsigned integer type (u8, u16, u32, u64, u128).
/// For example, a u16:
/// ```
/// use splitbits::splitbits;
///
/// let input: u16 = 0b1111_0000_1010_0011;
/// // Note how you can insert spaces wherever you like in the template without affecting meaning.
/// let nibbles = splitbits!(input, "aaaa bbbb cccc dddd");
/// assert_eq!(nibbles.a, 0b1111u8);
/// assert_eq!(nibbles.b, 0b0000u8);
/// assert_eq!(nibbles.c, 0b1010u8);
/// assert_eq!(nibbles.d, 0b0011u8);
/// ```
/// (If you need non-standard width integers (e.g. `u7`, `u1`, `u39`) , see [`splitbits_ux!`])
///
/// By default, each field will be assigned the smallest type that will fit it. To override this
/// behavior, use the min setting (valid options: `bool`, `u8`, `u16`, `u32`, `u64`, and `u128`):
/// ```
/// use splitbits::splitbits;
///
/// let input: u32 = 0b11110000_10100011_11110000_10100011;
/// // Note how you can insert spaces wherever you like in the template without affecting meaning.
/// let fields = splitbits!(min=u16, input, "aaaaaaaa bbbbbbbb bbbbbbbb ccdddddd");
/// assert_eq!(fields.a, 0b1111_0000u16);
/// assert_eq!(fields.b, 0b10100011_11110000u16);
/// assert_eq!(fields.c, 0b10u16);
/// assert_eq!(fields.d, 0b10_0011u16);
/// ```
///
/// By default, single-bit fields are extracted as booleans (1 = true, 0 = false):
/// ```
/// use splitbits::splitbits;
///
/// let fields = splitbits!(0b10111010, "beefyman");
/// assert_eq!(fields.b, true);
/// assert_eq!(fields.e, 0b01);
/// assert_eq!(fields.f, true);
/// assert_eq!(fields.y, true);
/// assert_eq!(fields.m, false);
/// assert_eq!(fields.a, true);
/// assert_eq!(fields.n, false);
/// ```
///
/// If you don't want any booleans, you can set the min setting to `u8` (or
/// higher):
/// ```
/// use splitbits::splitbits;
///
/// let fields = splitbits!(min=u8, 0b10111010, "beefyman");
/// assert_eq!(fields.b, 1);
/// assert_eq!(fields.e, 0b01);
/// assert_eq!(fields.f, 1);
/// assert_eq!(fields.y, 1);
/// assert_eq!(fields.m, 0);
/// assert_eq!(fields.a, 1);
/// assert_eq!(fields.n, 0);
/// ```
/// (If you want `u1`s instead of `bool`s, see [`splitbits_ux!`])
///
/// To ignore certain bits, use periods as placeholders:
/// ```
/// use splitbits::splitbits;
///
/// let letters = splitbits!(0b11000011, "aabb..zz");
/// assert_eq!(letters.a, 0b11);
/// assert_eq!(letters.b, 0b00);
/// assert_eq!(letters.z, 0b11);
/// ```
///
/// A field may exist as multiple segments in a template:
/// ```
/// use splitbits::splitbits;
///
/// let coordinates = splitbits!(0b11000011, "xxyyyyxx");
/// assert_eq!(coordinates.x, 0b1111);
/// assert_eq!(coordinates.y, 0b0000);
/// ```
#[proc_macro]
pub fn splitbits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Binary, Precision::Standard)
}

/// Same as [`splitbits!`], except that the widths of the generated fields are precise to-the-bit.
/// A dependency on the ux crate is required.
/// ```
/// use splitbits::splitbits_ux;
/// use ux::{u3, u5};
///
/// let fields = splitbits_ux!(0b11110000, "aaabbbbb");
/// // Single-letter field names, directly from the unique letters in the template above.
/// assert_eq!(fields.a, u3::new(0b111));
/// assert_eq!(fields.b, u5::new(0b10000));
/// ```
///
/// The min setting determines the smallest type to store a field.
/// It can be any value from `u1` to `u128` (though the default is `bool`):
/// ```
/// use splitbits::splitbits_ux;
/// use ux::u6;
///
/// let fields = splitbits_ux!(min=u6, 0b11110000, "aaabbbbb");
/// // Single-letter field names, directly from the unique letters in the template above.
/// assert_eq!(fields.a, u6::new(0b111));
/// assert_eq!(fields.b, u6::new(0b10000));
/// ```
///
/// To prevent `bool`s from being used, set min to `u1`:
/// ```
/// use splitbits::splitbits_ux;
/// use ux::{u1, u2};
///
/// let fields = splitbits_ux!(min=u1, 0b10111010, "beefyman");
/// assert_eq!(fields.b, u1::new(1));
/// assert_eq!(fields.e, u2::new(0b01));
/// assert_eq!(fields.f, u1::new(1));
/// assert_eq!(fields.y, u1::new(1));
/// assert_eq!(fields.m, u1::new(0));
/// assert_eq!(fields.a, u1::new(1));
/// assert_eq!(fields.n, u1::new(0));
/// ```
#[proc_macro]
pub fn splitbits_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Binary, Precision::Ux)
}

/// Same as [`splitbits!`], except that the template characters represent hexadecimal digits.
/// ```
/// use splitbits::splithex;
///
/// // Parse an IPV6 address.
/// let groups = splithex!(
///        0x2001_0db8_85a3_0000_0000_8a2e_0370_7334,
///         "aaaa bbbb cccc dddd eeee ffff gggg hhhh",
/// );
/// assert_eq!(groups.a, 0x2001u16);
/// assert_eq!(groups.b, 0x0db8u16);
/// assert_eq!(groups.c, 0x85a3u16);
/// assert_eq!(groups.d, 0x0000u16);
/// assert_eq!(groups.e, 0x0000u16);
/// assert_eq!(groups.f, 0x8a2eu16);
/// assert_eq!(groups.g, 0x0370u16);
/// assert_eq!(groups.h, 0x7334u16);
/// ```
///
/// Placeholders for hexadecimal macros ignore 4 bits, not just 1:
/// ```
/// use splitbits::splithex;
///
/// let fields = splithex!(0xABCDEF01, "xxx..y..");
/// assert_eq!(fields.x, 0xABC);
/// assert_eq!(fields.y, 0xF);
/// ```
///
/// Using the min setting:
/// ```
/// use splitbits::splithex;
///
/// let fields = splithex!(
///        min=u64,
///        0x2F010DB8_85A30000,
///         "abbccccc zzzzzzzz",
/// );
/// assert_eq!(fields.a, 0x2u64);
/// assert_eq!(fields.b, 0xF0u64);
/// assert_eq!(fields.c, 0x10DB8u64);
/// assert_eq!(fields.z, 0x85A30000u64);
/// ```
#[proc_macro]
pub fn splithex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Hexadecimal, Precision::Standard)
}

/// Same as [`splithex!`], except that the widths of the generated fields are precise to-the-bit.
/// A dependency on the ux crate is required.
/// ```
/// use splitbits::splithex_ux;
/// use ux::{u4, u12, u24};
///
/// // Parse an IPV6 address.
/// let fields = splithex_ux!(
///        0x2F010DB8_85A30000,
///         "abbccc.. ..zzzzzz",
/// );
/// assert_eq!(fields.a, u4::new(0x2));
/// assert_eq!(fields.b, 0xF0u8);
/// assert_eq!(fields.c, u12::new(0x10D));
/// assert_eq!(fields.z, u24::new(0xA30000));
/// ```
///
/// Using the min setting:
/// ```
/// use splitbits::splithex_ux;
/// use ux::{u13, u24};
///
/// let fields = splithex_ux!(
///        min=u13,
///        0x2F010DB8_85A30000,
///         "abbccc.. ..zzzzzz",
/// );
/// assert_eq!(fields.a, u13::new(0x2));
/// assert_eq!(fields.b, u13::new(0xF0));
/// assert_eq!(fields.c, u13::new(0x10D));
/// assert_eq!(fields.z, u24::new(0xA30000));
/// ```
#[proc_macro]
pub fn splithex_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_base(input, Base::Hexadecimal, Precision::Ux)
}

/// Same as [`splitbits!`], except that full-length variable names can be used. Returns a tuple
/// instead of a generated struct. If there is only a single field specified in the template,
/// returns a single variable instead (not a 1-tuple). Fields are returned in the order that they
/// appear in the template, and the single character template names are discarded.
/// ```
/// use splitbits::splitbits_named;
///
/// let (apple_count, banana_count) = splitbits_named!(0b11110000, "aaabbbbb");
/// assert_eq!(apple_count, 0b111);
/// assert_eq!(banana_count, 0b10000);
/// ```
///
/// Existing variables can be set, rather than declaring new ones:
/// ```
/// use splitbits::splitbits_named;
///
/// let mut apple_count = 5;
/// let banana_count;
///
/// /* Various operations on apple_count and banana_count omitted here. */
///
/// // Overwrite the existing values of apple_count and banana_count.
/// (apple_count, banana_count) = splitbits_named!(0b11110000, "aaabbbbb");
/// assert_eq!(apple_count, 0b111);
/// assert_eq!(banana_count, 0b10000);
/// ```
///
/// Just as with `[splitbits!]`, the template can have spaces for readability, period placeholders
/// for ignoring certain bits, and fields broken up into multiple segments:
/// ```
/// use splitbits::splitbits_named;
///
/// let input = 0b1111_0000;
/// let (apple_count, banana_count) = splitbits_named!(min=u32, input, "a b.b. aaa");
/// assert_eq!(apple_count, 0b1000u32);
/// assert_eq!(banana_count, 0b11u32);
/// ```
#[proc_macro]
pub fn splitbits_named(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_base(input, Base::Binary, Precision::Standard)
}

/// Same as [`splitbits_named!`], except that the widths of the generated fields are precise to-the-bit.
/// A dependency on the ux crate is required.
/// ```
/// use splitbits::splitbits_named_ux;
/// use ux::{u3, u5};
///
/// let (apple_count, banana_count) = splitbits_named_ux!(0b11110000, "aaabbbbb");
/// assert_eq!(apple_count, u3::new(0b111));
/// assert_eq!(banana_count, u5::new(0b10000));
/// ```
#[proc_macro]
pub fn splitbits_named_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_base(input, Base::Binary, Precision::Ux)
}

/// Same as [`splitbits_named!`] except with hexadecimal digits in the template.
/// ```
/// use splitbits::splithex_named;
///
/// let (zebras, bees, beavers, fish) = splithex_named!(
///     0x2F010DB8_85A30000,
///      "zbbvvvvv ffffffff",
/// );
/// assert_eq!(zebras,  0x2);
/// assert_eq!(bees,    0xF0);
/// assert_eq!(beavers, 0x10DB8);
/// assert_eq!(fish,    0x85A30000);
/// ```
#[proc_macro]
pub fn splithex_named(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_base(input, Base::Hexadecimal, Precision::Standard)
}

/// Same as [`splithex_named!`], except that the widths of the generated fields are precise
/// to-the-bit. A dependency on the ux crate is required.
/// ```
/// use splitbits::splithex_named_ux;
/// use ux::{u4, u20};
///
/// let (zebras, bees, beavers, fish) = splithex_named_ux!(
///     0x2F010DB8_85A30000,
///      "zbbvvvvv ffffffff",
/// );
/// assert_eq!(zebras,  u4::new(0x2));
/// assert_eq!(bees,    0xF0u8);
/// assert_eq!(beavers, u20::new(0x10DB8));
/// assert_eq!(fish,    0x85A30000u32);
/// ```
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
    replacebits_base(input, Base::Binary)
}

#[proc_macro]
pub fn replacehex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    replacebits_base(input, Base::Hexadecimal)
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

    let mut on_overflow = OnOverflow::Truncate;
    if parts.len() == 3 {
        let (setting, value) = parse_assignment(&parts[0])
            .expect("the first argument to be an 'overflow' setting since three arguments were supplied");
        assert_eq!(setting, "overflow", "Only 'overflow' is allowed as a setting.");
        on_overflow = OnOverflow::parse(&value)
            .unwrap_or_else(|err_string| panic!("Invalid type for setting 'overflow'. {err_string}"));

        parts.remove(0);
    }

    let value = parts[0].clone();
    let template = Template::from_expr(&parts[1], base, Precision::Ux);
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
