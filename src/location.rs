use proc_macro2::TokenStream;
use quote::quote;

use crate::name::Name;
use crate::Type;

#[derive(Clone, Copy)]
pub struct Location {
    pub len: u8,
    pub mask_offset: u8,
}

impl Location {
    pub const fn len(self) -> u8 {
        self.len
    }

    pub const fn mask_offset(self) -> u8 {
        self.mask_offset
    }

    pub fn to_mask(self) -> u128 {
        self.to_unshifted_mask() << self.mask_offset
    }

    pub fn to_unshifted_mask(self) -> u128 {
        2u128.pow(u32::from(self.len)) - 1
    }

    pub fn place_field_name(self, name: Name, width: Type, on_overflow: OnOverflow) -> TokenStream {
        let width = width.to_token_stream();
        let var = name.to_char();
        let name = name.to_ident();
        let shift = self.mask_offset();
        let mask = self.to_unshifted_mask();
        let len = self.len();
        match on_overflow {
            OnOverflow::Corrupt  => quote! { #width::from(#name) << #shift },
            OnOverflow::Shrink   => quote! { (#width::from(#name) & (#mask as #width)) << #shift },
            OnOverflow::Panic    => quote! {
                {
                    let n = #width::from(#name);
                    assert!(n <= #mask as #width,
                        "Variable {} is too big for its location in the template. {n} > {} ({} bits)", #var, #mask, #len);
                    n << #shift
                }
            },
            OnOverflow::Saturate => quote! {
                {
                    let mut n = #width::from(#name);
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
    Shrink,
    // Panic if the field is too large for its slot.
    Panic,
    // Allow oversized fields to corrupt the bits before them.
    Corrupt,
    // Set all bits in the slot to 1s if the field is too large.
    Saturate,
}

impl OnOverflow {
    pub fn parse(text: &str) -> Result<OnOverflow, String> {
        Ok(match text {
            "shrink" => OnOverflow::Shrink,
            "panic" => OnOverflow::Panic,
            "corrupt" => OnOverflow::Corrupt,
            "saturate" => OnOverflow::Saturate,
            overflow => return Err(
                format!("'{overflow}' is an invalid overflow option. Options: 'wrap', 'panic', 'corrupt', 'saturate'.")),
        })
    }
}
