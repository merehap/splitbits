use std::fmt;

use proc_macro2::TokenStream;
use quote::{quote, format_ident};

/* What type is associated with a Field, field Segment, or Template.
 * Can be a numeric type or a boolean.
 * There are two 1-bit types: bool and u1.
 */
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Type {
    Bool,
    Num(BitCount),
}

impl Type {
    // Create a valid Type for a Template given a template width.
    pub fn for_template(bit_count: u8) -> Self {
        match bit_count {
            8 | 16 | 32 | 64 | 128 => Self::Num(BitCount::new(bit_count).unwrap()),
            _ => panic!("Template width must be 8, 16, 32, 64, or 128, but was {bit_count}."),
        }
    }

    /* Determine what Type is needed for a Field that has the specified width and Precision.
     * Ux Precision means the Type will exactly match the width.
     * Standard Precision will result in the Type being the smallest built-in integer type that is
     * equal to or larger than the specified bit_count.
     */
    pub fn for_field(bit_count: u8, precision: Precision) -> Self {
        match bit_count {
            0 => panic!(),
            1 => Self::Bool,
            1..=128 if precision == Precision::Ux => Self::Num(BitCount::new(bit_count).unwrap()),
            2..=8    => Self::Num(BitCount::U8),
            9..=16   => Self::Num(BitCount::U16),
            17..=32  => Self::Num(BitCount::U32),
            33..=64  => Self::Num(BitCount::U64),
            65..=128 => Self::Num(BitCount::U128),
            129..=u8::MAX => panic!("Integers larger than u128 are not supported."),
        }
    }

    // Return true if the Type corresponds to a built-in type (bool, u8, u16, u32, u64, u128).
    pub const fn is_standard(self) -> bool {
        match self {
            Self::Bool => true,
            Self::Num(n) => matches!(n.0, 8 | 16 | 32 | 64 | 128),
        }
    }

    // Combine two Types to create a larger, Standard-precision type.
    pub fn concat(self, other: Self) -> Self {
        Self::for_field(self.bit_count() + other.bit_count(), Precision::Standard)
    }

    // Convert the Type to how it will appear in the macro expansion (e.g. bool, u7, u32).
    pub fn to_token_stream(self) -> TokenStream {
        let ident = format_ident!("{}", self.to_string());
        if self.is_standard() {
            quote! { #ident }
        } else {
            quote! { ux::#ident }
        }
    }

    // How many bits the Type corresponds to.
    pub const fn bit_count(self) -> u8 {
        match self {
            Self::Bool => 1,
            Self::Num(n) => n.0,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Bool => write!(f, "bool"),
            Self::Num(n) => write!(f, "u{}", n.0),
        }
    }
}

// Whether only bool, u8, u16, u32, u64, and u128 should be used, or if any ux types up to u127 are
// allowable too.
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Precision {
    Standard,
    Ux,
}

// Allowable bit counts for a Type. Anything from 1 to 128.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct BitCount(u8);

impl BitCount {
    pub const U8: Self = Self(8);
    pub const U16: Self = Self(16);
    pub const U32: Self = Self(32);
    pub const U64: Self = Self(64);
    pub const U128: Self = Self(128);

    // Create a new BitCount, failing if 0 or >128 are provided.
    pub fn new(count: u8) -> Result<Self, String> {
        match count {
            0 => Err("u0 is not a valid type of integer. BitCount must be positive.".into()),
            1..=128 => Ok(Self(count)),
            129..=u8::MAX => Err("Integers larger than u128 are not supported.".into()),
        }
    }
}
