//! [![github]](https://github.com/merehap/splitbits)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//!
//! Splitbits provides concise macros for extracting bits from integers and combining bits into
//! integers.
//! ```
//! use splitbits::splitbits;
//!
//! // Parse the template provided ("aaabbbbb"), apply it to the input, then generate a struct
//! // populated with the bit field values.
//! let fields = splitbits!(0b11110000, "aaabbbbb");
//! // Single-letter field names, generated from the unique letters in the template above.
//! assert_eq!(fields.a, 0b111);
//! assert_eq!(fields.b, 0b10000);
//! ```
//!
//! # Why use splitbits?
//! Splitbits allows you to skip tedious, error-prone bit operations, instead providing a
//! simple, terse, and readable template format for specifying which bits correspond to which
//! fields.
//!
//! Splitbits is intended for cases where [bitfield] is too heavy-weight: when you don't want to
//! explicitly declare a new struct for data that you won't use as a return value or argument.
//! Splitbits also provides some features that are arguably out of scope for [bitfield].
//!
//! [bitfield]: https://docs.rs/bitfield/latest/bitfield/
//!
//! # The four base macros
//! For additional examples, see each macro's page.
//! - [`splitbits!`] - Extract bit fields out of an integer, storing them as fields of a struct.
//! (See example above.)
//! - [`combinebits!`] - Combine bits of multiple integers into a single integer.
//!   ```
//!   use splitbits::combinebits;
//!
//!   let b: u8 = 0b1010_1010;
//!   let m: u8 = 0b1111;
//!   let e: u8 = 0b0000;
//!   let result = combinebits!("bbbb bbbb mmmm eeee");
//!   assert_eq!(result,       0b1010_1010_1111_0000);
//!   ```
//! - [`splitbits_then_combine!`] - Extract bit fields from multiple integers then combine them
//! into a single integer.
//!   ```
//!   use splitbits::splitbits_then_combine;
//!
//!   let output = splitbits_then_combine!(
//!       0b1111_0000, "aaaa ..bb", // input 0, input template 0,
//!       0b1011_1111, "cc.. ....", // input 1, input template 1
//!                    "aaaa bbcc", // output template
//!   );
//!   assert_eq!(output, 0b1111_0010);
//!   ```
//! - [`replacebits!`] - Replace some of the bits of an integer with bits from other integers.
//!   ```
//!   use splitbits::replacebits;
//!
//!   let a: u16 = 0b101;
//!   let b: u8 = 0b01;
//!   // Placeholder periods in the template are the bits that will not be replaced.
//!   let result = replacebits!(0b10000001, "aaa..bb.");
//!   assert_eq!(result,                   0b10100011);
//!   ```
//!
//! # Macro variants
//! The four base macros cover all the basic functionality that this crate offers and should be
//! sufficient for most use-cases. However, in many situations better ergonomics can be achieved by
//! using the macro variants.
//! #### Hexadecimal
//! All four base macros have equivalents that use hexadecimal digits for their templates rather
//! than bits (binary digits). The variants are [`splithex!`], [`combinehex!`],
//! [`splithex_then_combine!`], and [`replacehex!`].
//!
//! #### Splitbits variants
//! [`splitbits!`] itself has many variants which are intended for better ergonomics for the generated
//! variables. The basic variants are:
//! - [`splitbits_named!`] - Used when single-letter variable names aren't descriptive enough. This
//! variant returns a tuple (instead of a struct) of the resulting fields, allowing the caller to
//! assign individual long field names in the `let` binding.
//! - [`splitbits_named_into!`] - Same as [`splitbits_named!`] except that the caller specifies the
//! types of the resulting fields, not just their names. `into()` is called on each tuple field
//! before it reaches the caller. This is useful for when the default type (the smallest integer
//! type that will fit the field) is a smaller type than the caller would like to use, or if the
//! caller has a newtype that they would like to use instead.
//! - [`splitbits_ux!`] - Used when exact-width integers (e.g. u4, u7, u20) are needed, instead of
//! just the standard types (u8, u16, u32, u64, u128, and bool). Requires the [ux] crate.
//!
//! [ux]: <https://docs.rs/ux/latest/ux/>
//! # Template syntax
//! Templates are a string of characters that represent the names and bit-placements of fields
//! within an integer.
//!
//! Example: `"..aa bccc dddd 1010"`
//!
//! The possible elements of a template are:
//! - Names - a single letter that indicates the name of a field. (Currently only ASCII allowed.)
//! - Placeholders - a period that indicates a digit that will be ignored.
//! - Literals - a literal digit of the numeric base of the template (e.g. binary or hexadecimal).
//! - Whitespaces - an empty space character used to make formatting more human-friendly,
//! paralleling how underscores can be added to integer literals.
//!
//! The bits of a field are usually contiguous within a template, but they don't have to be:
//! `"aabbbbaa"`. This template will interpret `a` as a single field, with no bits present between
//! the halves.
//!
//! #### Restrictions
//! - Templates (currently) must have a standard integer width (8, 16, 32, 64, or 128 bits).
//! - Placeholders cannot be used in the template for [`combinebits!`], nor in the output template
//! of [`splitbits_then_combine!`]. They are not meaningful in those contexts.
//! - Literals (currently) cannot be used in the template for [`splitbits!`] nor the input templates
//! of [`splitbits_then_combine!`]. In the future, literals could be used in these contexts for
//! input validation.
//!
//! # Settings
//! There are currently two settings that can be passed to change the behavior of the various
//! macros:
//! - **min** - sets the minimum size of variable that can be produced by the [`splitbits!`] family of
//! macros. Must be set if you don't want booleans generated for 1-bit fields.
//!   - For standard (non-ux) macros, the valid setting values are `bool` (the default), `u8`, `u16`, `u32`,
//! `u64`, and `u128`. See examples at [`splitbits!`].
//!   - For ux macros, the valid setting values are `bool` (the default) or `uX`, where X is
//!   between 1 and 128. See examples at [`splitbits_ux!`].
//! - **overflow** - sets the behavior to use if the value of an input variable is larger than the
//! corresponding slot in the template. Used in [`combinebits!`] and [`replacebits!`]. Valid
//! setting values are `truncate` (the default), `panic`, `corrupt`, or `saturate`.

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
// * Fix combinebits! from failing when the template width is less than an input width.
// * Put compile checks behind different target so compiler updates don't break building.
// * Add missing variable test for splitbits.
// * Add wrong number of args test for splitbits.
// * Incorrect template size test.
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
/// If single-letter variable names aren't good enough, see [`splitbits_named!`]
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
///
/// [`splitbits!`] generates unique, undocumented, struct names. Changes to the struct name format
/// will not be considered breaking changes, so don't rely on the format staying the same!
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
/// first appear in the template, and the single character template names are discarded.
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
/// /* Various operations on apple_count omitted here. */
///
/// // Overwrite the existing values of apple_count and banana_count.
/// (apple_count, banana_count) = splitbits_named!(0b11110000, "aaabbbbb");
/// assert_eq!(apple_count, 0b111);
/// assert_eq!(banana_count, 0b10000);
/// ```
///
/// Just as with [`splitbits!`], the template can have spaces for readability, period placeholders
/// for ignoring certain bits, and fields broken up into multiple segments:
/// ```
/// use splitbits::splitbits_named;
///
/// let input = 0b1111_0000;
/// let (apple_count, banana_count) = splitbits_named!(min=u32, input, "ab.b .aaa");
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

/// Same as [`splitbits_named!`], except the caller can provide the field types, rather than the
/// macro inferring them. The custom types must implement From for the relevant integer types.
/// ```
/// use splitbits::splitbits_named_into;
///
/// let (apple_count, banana_count): (u32, u8) = splitbits_named_into!(0b11110000, "aaabbbbb");
/// assert_eq!(apple_count, 0b111);
/// assert_eq!(banana_count, 0b10000);
/// ```
///
/// Splitting into custom defined types:
///
/// Limitation - A From impl for the custom type must exist from the smallest integer type that
/// will fit the field. For example, for AppleCount below, which wraps a `u32`, `impl From<u32> for
/// AppleCount` won't work since "a" is first inferred as a `u8` (not a `u32`).
/// ```
/// use splitbits::splitbits_named_into;
///
/// let (apple_count, banana_count): (AppleCount, BananaCount) =
///     splitbits_named_into!(0b11110000, "aaabbbbb");
/// assert_eq!(apple_count, AppleCount(0b111u32));
/// assert_eq!(banana_count, BananaCount(0b10000u8));
///
/// #[derive(PartialEq, Debug)]
/// struct AppleCount(u32);
///
/// impl From<u8> for AppleCount {
///     fn from(value: u8) -> Self {
///         Self(value.into())
///     }
/// }
///
/// #[derive(PartialEq, Debug)]
/// struct BananaCount(u8);
///
/// impl From<u8> for BananaCount {
///     fn from(value: u8) -> Self {
///         Self(value)
///     }
/// }
/// ```
///
/// Declaring the fields and their types separate from initialization:
/// ```
/// use splitbits::splitbits_named_into;
///
/// let apple_count: u32;
/// let mut banana_count: u8 = 3;
/// (apple_count, banana_count) = splitbits_named_into!(0b11110000, "aaabbbbb");
/// assert_eq!(apple_count, 0b111);
/// assert_eq!(banana_count, 0b10000);
/// ```
#[proc_macro]
pub fn splitbits_named_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_into_base(input, Base::Binary, Precision::Standard)
}

/// Same as [`splitbits_named_into!`], except that the widths of the generated fields are precise
/// to-the-bit. A dependency on the ux crate is required.
/// ```
/// use splitbits::splitbits_named_into_ux;
/// use ux::{u6, u10};
///
/// let (apple_count, banana_count): (u10, u6) = splitbits_named_into_ux!(0b11110000, "aaabbbbb");
/// assert_eq!(apple_count, u10::new(0b111));
/// assert_eq!(banana_count, u6::new(0b10000));
/// ```
///
/// Splitting into custom defined types:
///
/// Limitation - A From impl for the custom type must exist from the relevant ux type. For example,
/// for AppleCount below, which wraps a `u32`, `impl From<u32> for AppleCount` won't work since "a"
/// is first inferred as a `u3` (not a `u32`).
/// ```
/// use splitbits::splitbits_named_into_ux;
/// use ux::{u3, u5};
///
/// let (apple_count, banana_count): (AppleCount, BananaCount) =
///     splitbits_named_into_ux!(0b11110000, "aaabbbbb");
/// assert_eq!(apple_count, AppleCount(0b111u32));
/// assert_eq!(banana_count, BananaCount(u5::new(0b10000)));
///
/// #[derive(PartialEq, Debug)]
/// struct AppleCount(u32);
///
/// impl From<u3> for AppleCount {
///     fn from(value: u3) -> Self {
///         Self(value.into())
///     }
/// }
///
/// #[derive(PartialEq, Debug)]
/// struct BananaCount(u5);
///
/// impl From<u5> for BananaCount {
///     fn from(value: u5) -> Self {
///         Self(value)
///     }
/// }
/// ```
#[proc_macro]
pub fn splitbits_named_into_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_into_base(input, Base::Binary, Precision::Ux)
}

/// Same as [`splithex_named!`], except the caller can provide the field types, rather than the
/// macro inferring them. The custom types must implement From/Into for the relevant integer types.
/// ```
/// use splitbits::splithex_named_into;
///
/// let (apple_count, banana_count): (u16, u32) = splithex_named_into!(0x89ABCDEF, "aaabbbbb");
/// assert_eq!(apple_count, 0x89A);
/// assert_eq!(banana_count, 0xBCDEF);
/// ```
///
/// See [`splitbits_named_into!`] for more examples.
#[proc_macro]
pub fn splithex_named_into(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_into_base(input, Base::Hexadecimal, Precision::Standard)
}

/// Same as [`splithex_named_into!`], except the widths of the generated fields are precise
/// to-the-bit.
/// ```
/// use splitbits::splithex_named_into_ux;
/// use ux::{u12, u20};
///
/// let (apple_count, banana_count): (u12, u20) = splithex_named_into_ux!(0x89ABCDEF, "aaabbbbb");
/// assert_eq!(apple_count, u12::new(0x89A));
/// assert_eq!(banana_count, u20::new(0xBCDEF));
/// ```
///
/// See [`splitbits_named_into_ux!`] for more examples.
#[proc_macro]
pub fn splithex_named_into_ux(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    splitbits_named_into_base(input, Base::Hexadecimal, Precision::Ux)
}

/// Combine bits of multiple variables into a single variable as defined by a template.
///
/// By default, input values that are too large for their slot in the template will have their
/// front bits truncated until they fit. See later examples for how to override this behavior.
/// ```
/// use splitbits::combinebits;
///
/// let b: u8 = 0b1010_1010;
/// let m: u8 = 0b1111;
/// let e: u8 = 0b0000;
/// let result = combinebits!("bbbb bbbb mmmm eeee");
/// assert_eq!(result,       0b1010_1010_1111_0000);
/// ```
///
/// If descriptive variable names are desired, then variables can be passed in as arguments.
/// These variables must occur in the same order in the argument list as the name characters occur
/// in the template. The single character template names are ignored beyond this.
/// ```
/// use splitbits::combinebits;
///
/// let beginning: u8 = 0b1010_1010;
/// let middle: u8 = 0b1111;
/// let end: u16 = 0b0000;
/// let result = combinebits!(beginning, middle, end, "bbbb bbbb mmmm eeee");
/// assert_eq!(result,                           0b1010_1010_1111_0000);
/// ```
///
/// An input variable can be split into multiple segments by the template:
/// ```
/// use splitbits::combinebits;
///
/// let e: u16 = 0b100000_0000001;
/// let m: u8 = 0b111;
/// let result = combinebits!("eeee eemm meee eeee");
/// assert_eq!(result,       0b1000_0011_1000_0001);
/// ```
///
/// Bits with a fixed (non-variable) value can be set explicitly in the template:
/// ```
/// use splitbits::combinebits;
///
/// let a: u8 = 0b10;
/// let b: u8 = 0b01;
/// let result = combinebits!("1100aabb");
/// assert_eq!(result,       0b11001001);
/// ```
///
/// Arbitrary-sized integers from the ux crate can be used as input variables:
/// ```
/// use splitbits::combinebits;
/// use ux::{u1, u3, u7};
///
/// let enabled = true;
/// let x_coord: u7 = u7::new(0b1100000);
/// let y_coord: u3 = u3::new(0b100);
/// let z_coord: u1 = u1::new(1);
/// let result = combinebits!(enabled, x_coord, y_coord, z_coord, "exxxxxxx yyyz0000");
/// assert_eq!(result,                                           0b11100000_10010000);
/// ```
///
/// If an input variable is too large for its slot, by default its top bits are truncated (but
/// other options exist):
/// ```
/// use splitbits::combinebits;
///
/// let a: u8 = 0b0110_0001;
/// // Equivalent of: (a & bitmask) << 1
/// let result = combinebits!("0aaaaaa0");
/// assert_eq!(result,       0b01000010);
/// ```
///
/// ```
/// use splitbits::combinebits;
///
/// // overflow=truncate is the default behavior, so the result is the same as above.
/// let a: u8 = 0b01100001;
/// // Equivalent of: (a & bitmask) << 1
/// let result = combinebits!(overflow=truncate, "0aaaaaa0");
/// assert_eq!(result,                          0b01000010);
/// ```
///
/// ```
/// use splitbits::combinebits;
///
/// // overflow=corrupt is the most efficient option, but corrupts the bits that preceed the
/// // field slot if an overflow occurs.
/// let a: u8 = 0b01100001;
/// // Equivalent of: a << 1
/// let result = combinebits!(overflow=corrupt, "0aaaaaa0");
/// assert_eq!(result,                         0b11000010);
/// ```
///
/// ```
/// use splitbits::combinebits;
///
/// // overflow=saturate sets all the bits of the field to 1s if an overflow occurs.
/// let a: u8 = 0b01100001;
/// // Equivalent of: min(a << 1, mask)
/// let result = combinebits!(overflow=saturate, "0aaaaaa0");
/// assert_eq!(result,                          0b01111110);
/// ```
///
/// ```should_panic
/// use splitbits::combinebits;
///
/// // overflow=panic results in a panic if the input variable doesn't fit in its template slot.
/// let a: u8 = 0b01100001;
/// // Equivalent of: assert!((a << 1) <= mask)
/// let _ = combinebits!(overflow=panic, "0aaaaaa0");
/// ```
#[proc_macro]
pub fn combinebits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    combinebits_base(input, Base::Binary)
}

/// Same as [`combinebits!`] except the template uses hexadecimal digits rather than binary digits.
///
/// Note that hexadecimal literals must be uppercase so that they don't conflict with field name
/// letters which must be lowercase.
/// ```
/// use splitbits::combinehex;
///
/// let b: u32 = 0x89ABCDEF;
/// let m: u8 = 0x11;
/// let e: u16 = 0x2345;
/// let result = combinehex!("bbbb bbbb mmAF eeee");
/// assert_eq!(result,      0x89AB_CDEF_11AF_2345);
/// ```
#[proc_macro]
pub fn combinehex(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    combinebits_base(input, Base::Hexadecimal)
}

/// Extract bits from multiple input integers by matching against input templates, then combine
/// those bits into to an integer matching the output template.
///
/// The width of a field must match between the input templates and output template.
///
/// Placeholder periods are usually needed with this macro for ignoring unused input bits.
/// ```
/// use splitbits::splitbits_then_combine;
///
/// // While its possible to use this macro on a single line, it's easiest to see the structure like
/// // this:
/// // let output = splitbits_then_combine!(
/// //    input0, input_template0,
/// //    input1, input_template1,
/// //    ...
/// //    inputX, input_templateX,
/// //            output_template,
/// // );
/// let output = splitbits_then_combine!(
///     0b1111_0000, "aaaa ..bb",
///     0b1011_1111, "cc.. ....",
///                  "aaaa bbcc",
/// );
/// assert_eq!(output, 0b1111_0010);
/// ```
///
/// ```
/// // Or the equivalent, with actual input variables, as it would occur in real code:
/// use splitbits::splitbits_then_combine;
///
/// let primary = 0b1111_0000;
/// let supplement = 0b1011_1111;
/// let output = splitbits_then_combine!(
///     primary,    "aaaa ..bb",
///     supplement, "cc.. ....",
///                 "aaaa bbcc",
/// );
/// assert_eq!(output, 0b1111_0010);
/// ```
///
/// Literal 1s and 0s can be hard-coded into the output template:
/// ```
/// use splitbits::splitbits_then_combine;
///
/// let output = splitbits_then_combine!(
///     0b1111_0000, "aaaa ..bb",
///     0b1011_1111, "cc.. ....",
///                  "aaaa 0101 0000 bbcc",
/// );
/// assert_eq!(output, 0b1111_0101_0000_0010);
/// ```
///
/// An input field can be split into segments by the output template:
/// ```
/// use splitbits::splitbits_then_combine;
///
/// let output = splitbits_then_combine!(
///     0b1111_0000, "aaaa aa..",
///     0b1011_1111, "..bb bbbb",
///                  "aaab bbbb b000 1aaa",
/// );
/// assert_eq!(output, 0b1111_1111_1000_1100);
/// ```
///
/// Segments of a field can be combined from different input locations into a single output field.
/// ```
/// use splitbits::splitbits_then_combine;
///
/// let output = splitbits_then_combine!(
///     0b1111_0000, "aaaa ..aa",
///     0b0011_1010, "..aa bbbb",
///                  "aaaa aaaa 0000 bbbb",
/// );
/// assert_eq!(output, 0b1111_0011_0000_1010);
/// ```
///
/// Having all these features in one macro means that there are multiple ways to achieve an
/// outcome, so consider which way leads to the best readability on a case-by-case basis.
#[proc_macro]
pub fn splitbits_then_combine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    split_then_combine_base(input, Base::Binary)
}

/// Same as [`splitbits_then_combine!`], except with hexadecimal digits in the template.
/// ```
/// use splitbits::splithex_then_combine;
///
/// let output = splithex_then_combine!(
///     0xCDEF_0000, "xxxx ..yy",
///     0xBA98_1111, "zz.. ....",
///                  "xxxx 0123 ABCD yyzz",
/// );
/// assert_eq!(output, 0xCDEF_0123_ABCD_00BA);
/// ```
#[proc_macro]
pub fn splithex_then_combine(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    split_then_combine_base(input, Base::Hexadecimal)
}

/// Replace the bits in an integer with bits from other variables, as specified by a template.
///
/// Limitation: Currently this macro doesn't take input variables as parameters, they must be
/// captured from single-letter variables.
/// ```
/// use splitbits::replacebits;
///
/// let a: u16 = 0b101;
/// let b: u8 = 0b00001;
/// let c: u128 = 0b0101;
/// let d = true;
///
/// let input = 0b1111_1111_0000_0000;
/// let output = replacebits!(input, "aaab bbbb .d.. cccc");
/// assert_eq!(output,              0b1010_0001_0100_0101);
/// ```
///
/// Literals can be placed in the template to override specific bits:
/// ```
/// use splitbits::replacebits;
///
/// let a: u16 = 0b101;
/// let b: u8 = 0b01;
/// let result = replacebits!(0b10000001, "aaa11bb0");
/// assert_eq!(result,                   0b10111010);
/// ```
///
/// Input variables can be split across multiple template slots:
/// ```
/// use splitbits::replacebits;
///
/// let a: u16 = 0b101101;
/// let b: u8 = 0b01;
/// let result = replacebits!(0b1111_1111, "aaab baaa");
/// assert_eq!(result,                    0b1010_1101);
/// ```
///
/// If an input variable is too large for its slot, by default its top bits are truncated (but
/// other options exist):
/// ```
/// use splitbits::replacebits;
///
/// let a: u8 = 0b01100001;
/// // Equivalent of: (a & mask) << 1
/// let result = replacebits!(0b00110000, "0aaaaaa0");
/// assert_eq!(result,                   0b01000010);
/// ```
///
/// ```
/// use splitbits::replacebits;
///
/// // overflow=truncate is the default behavior, so the result is the same as above.
/// let a: u8 = 0b01100001;
/// // Equivalent of: (a & mask) << 1
/// let result = replacebits!(overflow=truncate, 0b00110000, "0aaaaaa0");
/// assert_eq!(result,                                      0b01000010);
/// ```
///
/// ```
/// use splitbits::replacebits;
///
/// // overflow=corrupt is the most efficient option, but corrupts the bits that preceed the
/// // field slot if an overflow occurs.
/// let a: u8 = 0b01100001;
/// // Equivalent of: a << 1
/// let result = replacebits!(overflow=corrupt, 0b00110000, "0aaaaaa0");
/// assert_eq!(result,                                     0b11000010);
/// ```
///
/// ```
/// use splitbits::replacebits;
///
/// // overflow=saturate sets all the bits of the field to 1s if an overflow occurs.
/// let a: u8 = 0b01100001;
/// // Equivalent of: min(a << 1, mask)
/// let result = replacebits!(overflow=saturate, 0b00110000, "0aaaaaa0");
/// assert_eq!(result,                                      0b01111110);
/// ```
///
/// ```should_panic
/// use splitbits::replacebits;
///
/// // overflow=panic results in a panic if the input variable doesn't fit in its template slot.
/// let a: u8 = 0b01100001;
/// // Equivalent of: assert!((a << 1) <= mask))
/// let _ = replacebits!(overflow=panic, 0b01100010, "0aaaaaa0");
/// ```
#[proc_macro]
pub fn replacebits(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    replacebits_base(input, Base::Binary)
}

/// Same as [`replacebits!`], except the digits in the template are hexadecimal rather than binary.
/// ```
/// use splitbits::replacehex;
///
/// let w = 0xABC;
/// let x = 0x01234;
/// let y = 0x9876;
/// let z: u8 = 5;
///
/// let input = 0x555_5555_5F55_5555;
/// let output = replacehex!(input, "wwwx xxxx .z.. yyyy");
/// assert_eq!(output,             0xABC0_1234_5555_9876);
/// ```
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

    let expr = parts.pop().unwrap();
    let template = Template::from_expr(&expr, base, Precision::Ux);
    if template.has_placeholders() {
        let bad_template = Template::template_string(&expr);
        panic!(
            "Template ({bad_template}) must not have placeholders (periods) in it. \
            Use literals instead as appropriate.");
    }

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

    for part in &parts {
        if parse_assignment(part).is_some() {
            panic!("Either an input or template was missing, but found a setting instead.");
        }
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
