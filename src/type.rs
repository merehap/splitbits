use std::fmt;

use proc_macro2::TokenStream;
use quote::{quote, format_ident};

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug)]
pub enum Type {
    Bool,
    Ux(u8),
}

impl Type {
    pub const BOOL: Type = Type::Bool;

    pub fn ux(len: u8) -> Type {
        match len {
            0 => panic!(),
            1..=128 => Type::ux(len.try_into().unwrap()),
            129..=u8::MAX => panic!("Integers larger than u128 are not supported."),
        }
    }

    pub fn for_template(len: u8) -> Type {
        match len {
            8 => Type::ux(8),
            16 => Type::ux(16),
            32 => Type::ux(32),
            64 => Type::ux(64),
            128 => Type::ux(128),
            len => panic!("Template length must be 8, 16, 32, 64, or 128, but was {len}."),
        }
    }

    pub fn for_field(len: u8, precision: Precision) -> Type {
        match len {
            0 => panic!(),
            1..=128 if precision == Precision::Ux => Type::ux(len.try_into().unwrap()),
            1        => Type::ux(1),
            2..=8    => Type::ux(8),
            9..=16   => Type::ux(16),
            17..=32  => Type::ux(32),
            33..=64  => Type::ux(64),
            65..=128 => Type::ux(128),
            129..=u8::MAX => panic!("Integers larger than u128 are not supported."),
        }
    }

    pub fn is_standard(self) -> bool {
        if let Type::Ux(ux) = self {
            matches!(ux, 1 | 8 | 16 | 32 | 64 | 128)
        } else {
            false
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
            Type::Ux(ux) => ux,
        }
    }
}

impl From<u8> for Type {
    fn from(value: u8) -> Type {
        Type::Ux(value)
    }
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Type::Bool => write!(f, "bool"),
            Type::Ux(ux) => write!(f, "u{}", ux)
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum Precision {
    Standard,
    Ux,
}
