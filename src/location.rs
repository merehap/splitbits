use proc_macro2::TokenStream;
use quote::quote;

use crate::Type;

// The location of a bit field segment within a template.
#[derive(Clone, Copy)]
pub struct Location {
    // How wide the Location is.
    pub width: u8,
    // How far from the low bit (right-most bit) of the template is the low bit of this Location.
    pub mask_offset: u8,
}

impl Location {
    // How wide the Location is.
    pub const fn width(self) -> u8 {
        self.width
    }

    // Where the Location starts within the Template.
    pub const fn mask_offset(self) -> u8 {
        self.mask_offset
    }

    // Convert the Location to a bit mask: '1's within the location, '0's without.
    pub fn to_mask(self) -> u128 {
        self.to_unshifted_mask() << self.mask_offset
    }

    // Convert the Location to a bit mask, but starting at the start of the template.
    pub fn to_unshifted_mask(self) -> u128 {
        2u128.pow(u32::from(self.width)) - 1
    }

    /* Place the name of a field within its appropriate location in the template,
     * using the specified OnOverflow behavior if it is too long.
     */
    pub fn place_field_segment(
        self,
        label: TokenStream,
        segment: TokenStream,
        width: Type,
        on_overflow: OnOverflow,
    ) -> TokenStream {
        let width = width.to_token_stream();
        let shift = self.mask_offset();
        let mask = self.to_unshifted_mask();
        match on_overflow {
            OnOverflow::Corrupt  => quote! { #width::from(#segment) << #shift },
            OnOverflow::Truncate => quote! { (#width::from(#segment) & (#mask as #width)) << #shift },
            OnOverflow::Panic    => quote! {
                {
                    let n = #width::from(#segment);
                    assert!(n <= #mask as #width,
                        "Variable {} is too big for its location in the template. 0b{n:b} > 0b{:b}",
                        #label, #mask);
                    n << #shift
                }
            },
            OnOverflow::Saturate => quote! {
                {
                    let mut n = #width::from(#segment);
                    let mask = #mask as #width;
                    if n > mask {
                        n = mask;
                    }
                    n << #shift
                }
            },
        }
    }
}

// What behavior to use if a field is too big for its template slot during substitution.
#[derive(Debug, Clone, Copy)]
pub enum OnOverflow {
    // Remove the upper bits that don't fit in the template slot.
    Truncate,
    // Panic if the field is too large for its slot.
    Panic,
    // Allow oversized fields to corrupt the bits before them.
    Corrupt,
    // Set all bits in the slot to 1s if the field is too large.
    Saturate,
}

impl OnOverflow {
    // Convert a lower-case str into its corresponding OnOverflow value.
    pub fn parse(text: &str) -> Result<OnOverflow, String> {
        Ok(match text {
            "truncate" => OnOverflow::Truncate,
            "panic" => OnOverflow::Panic,
            "corrupt" => OnOverflow::Corrupt,
            "saturate" => OnOverflow::Saturate,
            overflow => return Err(format!("'{overflow}' is an invalid overflow option. \
                Options: 'truncate', 'panic', 'corrupt', 'saturate'.")),
        })
    }
}
