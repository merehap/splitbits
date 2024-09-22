use std::cmp::Ordering;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

use crate::location::Location;
use crate::r#type::Type;

// A portion of a Field within a Template.
#[derive(Clone)]
pub struct Segment {
    input: Expr,
    t: Type,
    location: Location,
    offset: u8,
}

impl Segment {
    // Create a new Segment from a syn Expr.
    pub const fn new(input: Expr, t: Type, location: Location, offset: u8) -> Self {
        Self { input, t, location, offset }
    }

    // Offset this Segment further within the Template.
    pub fn set_output_offset(&mut self, output_offset: u8) -> Self {
        self.offset += output_offset;
        self.clone()
    }

    // Increase how much space the Segment takes up, without changing its value Expr.
    pub fn widen(&mut self, new_type: Type) -> Self {
        if new_type > self.t {
            self.t = new_type;
        }

        self.clone()
    }

    /* Convert the Segment into how it will appear in the macro expansion.
     * It may have a left shift, a right shift, or no shift.
     */
    pub fn to_token_stream(&self) -> TokenStream {
        let input = &self.input;
        let ordering = self.shift().cmp(&0);
        let shift = self.shift().abs();
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

    // The width of the segment.
    pub const fn width(&self) -> u8 {
        self.location.width()
    }

    fn shift(&self) -> i16 {
        i16::from(self.location.mask_offset()) - i16::from(self.offset)
    }
}
