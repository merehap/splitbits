use std::fmt;

use proc_macro2::TokenStream;
use quote::{quote, format_ident};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Type {
    Bool,
    Num(BitCount),
}

impl Type {
    pub fn for_template(len: u8) -> Type {
        let bit_count = BitCount::new(len).unwrap();
        match len {
            8 | 16 | 32 | 64 | 128 => Type::Num(bit_count),
            len => panic!("Template length must be 8, 16, 32, 64, or 128, but was {len}."),
        }
    }

    pub fn for_field(len: u8, precision: Precision) -> Type {
        match len {
            0 => panic!(),
            1 => Type::Bool,
            1..=128 if precision == Precision::Ux => Type::Num(BitCount::new(len).unwrap()),
            2..=8    => Type::Num(BitCount::U8),
            9..=16   => Type::Num(BitCount::U16),
            17..=32  => Type::Num(BitCount::U32),
            33..=64  => Type::Num(BitCount::U64),
            65..=128 => Type::Num(BitCount::U128),
            129..=u8::MAX => panic!("Integers larger than u128 are not supported."),
        }
    }

    pub fn is_standard(self) -> bool {
        match self {
            Type::Bool => true,
            Type::Num(n) => matches!(n.0, 8 | 16 | 32 | 64 | 128),
        }
    }

    pub fn concat(self, other: Type) -> Type {
        Type::for_field(self.size() + other.size(), Precision::Standard)
    }

    pub fn to_token_stream(self) -> TokenStream {
        let ident = format_ident!("{}", self.to_string());
        if self.is_standard() {
            quote! { #ident }
        } else {
            quote! { ux::#ident }
        }
    }

    pub fn size(self) -> u8 {
        match self {
            Type::Bool => 1,
            Type::Num(n) => n.0,
        }
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Type::Bool => write!(f, "bool"),
            Type::Num(n) => write!(f, "u{}", n.0),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Precision {
    Standard,
    Ux,
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub struct BitCount(u8);

impl BitCount {
    pub const U8: BitCount = BitCount(8);
    pub const U16: BitCount = BitCount(16);
    pub const U32: BitCount = BitCount(32);
    pub const U64: BitCount = BitCount(64);
    pub const U128: BitCount = BitCount(128);

    pub fn new(count: u8) -> Result<Self, String> {
        match count {
            0 => Err("u0 is not a valid type of integer. BitCount must be positive.".into()),
            1..=128 => Ok(Self(count)),
            129..=u8::MAX => Err("Integers larger than u128 are not supported.".into()),
        }
    }
}
