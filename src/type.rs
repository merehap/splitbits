use std::fmt;

use proc_macro2::TokenStream;
use quote::{quote, format_ident};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Type(u8);

impl Type {
    pub const BOOL: Type = Type(1);

    pub fn for_template(len: u8) -> Type {
        match len {
            8 => Type(8),
            16 => Type(16),
            32 => Type(32),
            64 => Type(64),
            128 => Type(128),
            len => panic!("Template length must be 8, 16, 32, 64, or 128, but was {len}."),
        }
    }

    pub fn for_field(len: u8, precision: Precision) -> Type {
        match len {
            0 => panic!(),
            1..=128 if precision == Precision::Ux => Type(len.try_into().unwrap()),
            1        => Type(1),
            2..=8    => Type(8),
            9..=16   => Type(16),
            17..=32  => Type(32),
            33..=64  => Type(64),
            65..=128 => Type(128),
            129..=u8::MAX => panic!("Integers larger than u128 are not supported."),
        }
    }

    pub fn concat(self, other: Type) -> Type {
        Type::for_field(self.0 + other.0, Precision::Standard)
    }

    pub fn to_token_stream(self) -> TokenStream {
        let ident = format_ident!("{}", self.to_string());
        match self.0 {
            1 | 8 | 16 | 32 | 64 | 128 => quote! { #ident },
            _ => quote! { ux::#ident },
        }
    }
}

impl From<u8> for Type {
    fn from(value: u8) -> Type {
        Type(value)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.0 == 1 {
            write!(f, "bool")
        } else {
            write!(f, "u{}", self.0)
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Precision {
    Standard,
    Ux,
}
