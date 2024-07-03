use std::cmp::Ordering;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

use crate::location::Location;
use crate::r#type::Type;

#[derive(Clone)]
pub struct Segment {
    input: Expr,
    t: Type,
    location: Location,
    shift: i16,
}

impl Segment {
    pub fn new(input: Expr, t: Type, location: Location) -> Self {
        Self { input, t, location, shift: 0 }
    }

    pub fn shift_left(&mut self, shift: u8) -> Self {
        self.shift -= i16::from(shift);
        self.clone()
    }

    pub fn shift_right(&mut self, shift: u8) -> Self {
        self.shift += i16::from(shift);
        self.clone()
    }

    pub fn widen(&mut self, new_type: Type) -> Self {
        if new_type > self.t {
            self.t = new_type;
        }

        self.clone()
    }

    pub fn to_value(&self) -> TokenStream {
        let input = &self.input;
        let ordering = self.shift.cmp(&0);
        let shift = self.shift.abs();
        let shifter = match ordering {
            // There's no need to shift if the shift is 0.
            Ordering::Equal => quote! { },
            Ordering::Greater => quote! { >> #shift },
            Ordering::Less => quote! { << #shift },
        };

        let t = self.t.to_token_stream();
        let mask = self.location.to_mask();
        quote! { (#input as #t & #mask as #t) #shifter }
    }

    pub fn len(&self) -> u8 {
        self.location.len()
    }
}
