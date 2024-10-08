use std::collections::BTreeMap;

use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

use crate::name::Name;
use crate::location::Location;
use crate::segment::Segment;
use crate::r#type::{Type, Precision};

/* A bit field to be extracted from input in accordance with a Template.
 * A Field can be split over multiple segments in different locations in the Template.
 */
#[derive(Clone)]
pub struct Field {
    name: Name,
    segments: Vec<Segment>,
    bit_width: Type,
}

impl Field {
    /* Create a new Field, given the input expression and the segment locations within the input.
     * By default, the Field will use the smallest type needed to store it. This can be overriden
     * by setting min_size.
     */
    pub fn new(
        name: Name,
        input_type: Type,
        input: &Expr,
        precision: Precision,
        min_size: Option<Type>,
        locations: &[Location],
    ) -> Self {
        // Create one Segment per Location, increasing the segment offset as it goes.
        let mut segment_offset = 0;
        let mut segments = Vec::new();
        for &location in locations {
            let segment = Segment::new(input.clone(), input_type, location, segment_offset);
            segment_offset += location.width();
            segments.push(segment);
        }

        // Determine the appropriate bit width needed for this Field.
        let bit_width = locations.iter()
            .map(|location| location.width())
            .sum();
        let mut bit_width = Type::for_field(bit_width, precision);
        if let Some(min_size) = min_size && min_size > bit_width {
            bit_width = min_size;
        }

        Self { name, segments, bit_width }
    }

    // Convert the Field into its macro expansion format, either "bool" or "uX".
    pub fn to_token_stream(&self) -> TokenStream {
        let t = self.bit_width.to_token_stream();
        let mut segments = self.segments.iter().map(Segment::to_token_stream);
        if self.bit_width == Type::Bool {
            let segment = segments.next().unwrap();
            quote! { (#segment) != 0 }
        } else {
            quote! { #t::try_from(#(#segments)|*).unwrap() }
        }
    }

    // Merge two collections of fields into one, removing duplicates.
    pub fn merge(upper: &[Self], lower: &[Self]) -> Vec<Self> {
        let lower_map: BTreeMap<_, _> = lower.iter()
            .map(|field| (field.name, field))
            .collect();
        let mut result = Vec::new();
        for u in upper {
            if let Some(l) = lower_map.get(&u.name) {
                result.push(u.concat(l));
            } else {
                result.push(u.clone());
            }
        }

        let upper_map: BTreeMap<_, _> = upper.iter()
            .map(|field| (field.name, field))
            .collect();
        // Only push Fields from the lower map if they weren't already pushed from the upper map.
        for l in lower {
            if !upper_map.contains_key(&l.name) {
                result.push(l.clone());
            }
        }

        result
    }

    // Combine two Fields into one (their names must match).
    pub fn concat(&self, lower: &Self) -> Self {
        assert_eq!(self.name, lower.name);

        let mut new_segments = Vec::new();
        // Shift all of the existing Segments to the left to make room for the new segments.
        for segment in &self.segments {
            let new_segment = segment.clone().set_output_offset(lower.width());
            new_segments.push(new_segment);
        }

        // Add all the new segments (they get to keep their original offsets).
        for segment in &lower.segments {
            new_segments.push(segment.clone());
        }

        Self {
            name: self.name,
            segments: new_segments,
            bit_width: self.bit_width.concat(lower.bit_width),
        }
    }

    // Widen all segments.
    pub fn widen(mut self, new_bit_width: Type) -> Self {
        self.bit_width = new_bit_width;
        for segment in &mut self.segments {
            segment.widen(new_bit_width);
        }

        self
    }

    // The Field's single-letter name.
    pub const fn name(&self) -> Name {
        self.name
    }

    // How many bits are contained in this Field.
    pub const fn bit_width(&self) -> Type {
        self.bit_width
    }

    // TODO: Determine how this is used differently from bit_width().
    pub fn width(&self) -> u8 {
        self.segments.iter()
            .map(Segment::width)
            .sum()
    }
}
