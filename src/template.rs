use std::collections::{BTreeSet, BTreeMap};

use itertools::Itertools;
use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::{Expr, Lit};

use crate::base::Base;
use crate::character::{Character, Characters};
use crate::field::Field;
use crate::location::Location;
use crate::name::Name;
use crate::location::OnOverflow;
use crate::r#type::{Type, Precision};

/* A sequence of characters use to match and extract bit fields from an integer,
 * or alternately to combine bit fields into an integer.
 * For example, "aaabbcdd" will extract variables a, b, c, and d from a byte (u8),
 * or can be used to combine variables a, b, c, and d into a byte.
 */
pub struct Template {
    // How many bits of input the template will match against.
    // Equal to the number of characters only in base 2.
    width: Type,
    // Whether the template will match arbitrary bit width fields, or just standard widths.
    precision: Precision,
    // The template-legal characters contained in this template, in order.
    characters: Characters,
    // The locations of the disjoint segments of each bit field, paired with the field name.
    locations_by_name: Vec<(Name, Vec<Location>)>,
}

impl Template {
    // Create a Template from a String-format macro expression.
    pub fn from_expr(expr: &Expr, base: Base, precision: Precision) -> Self {
        let template_string = Self::template_string(expr);
        reject_higher_base_chars(&template_string, base);
        let characters = Characters::from_str(&template_string, base);

        let len: u8 = characters.len();
        let width = Type::for_template(len);
        let mut locations_by_name: Vec<(Name, Vec<Location>)> = Vec::new();
        for name in characters.to_names() {
            let locations: Vec<Location> = characters.iter()
                .rev()
                .enumerate()
                .chunk_by(|(_, &n)| n)
                .into_iter()
                .filter_map(|(c, segment)| {
                    if c == Character::Name(name) {
                        let segment: Vec<_> = segment.collect();
                        let len = segment.len().try_into().unwrap();
                        let mask_offset = segment[0].0.try_into().unwrap();
                        Some(Location { len, mask_offset })
                    } else {
                        None
                    }
                })
                .collect();
            locations_by_name.push((name, locations));
        }

        Template { width, precision, characters, locations_by_name }
    }

    // Extract the bit fields, as specified by the template, from the input expression.
    // Upsize any fields to min_size.
    pub fn extract_fields(&self, input: &Expr, min_size: Option<Type>) -> Vec<Field> {
        self.locations_by_name.iter()
            .map(|(name, locations)| Field::new(*name, self.width, input, self.precision, min_size, locations))
            .collect()
    }

    // Capture variables from outside the the macro, substituting them into the template.
    pub fn combine_variables(&self, on_overflow: OnOverflow) -> TokenStream {
        let mut field_streams = Vec::new();
        for (name, locations) in &self.locations_by_name {
            assert_eq!(locations.len(), 1);
            field_streams.push(locations[0].place_field_name(*name, self.width, on_overflow));
        }

        let mut literal_quote = quote! {};
        if let Some(literal) = self.characters.extract_literal() {
            let t = self.width.to_token_stream();
            literal_quote = quote! { | (#literal as #t) };
        }

        quote! { (#(#field_streams)|*) #literal_quote }
    }

    // Substitute macro arguments into the template.
    pub fn combine_with(&self, exprs: &[Expr]) -> TokenStream {
        let t = self.width.to_token_stream();
        let mut field_streams = Vec::new();
        assert_eq!(exprs.len(), self.locations_by_name.len(),
            "The number of inputs must be equal to the number of names in the template.",
        );
        for ((_name, locations), expr) in self.locations_by_name.iter().zip(exprs.iter()) {
            assert_eq!(locations.len(), 1);
            let shift = locations[0].mask_offset();
            let field = quote! { #t::from(#expr) << #shift };
            field_streams.push(field);
        }

        let mut literal_quote = quote! {};
        if let Some(literal) = self.characters.extract_literal() {
            let t = self.width.to_token_stream();
            literal_quote = quote! { | (#literal as #t) };
        }

        quote! { (#(#field_streams)|*) #literal_quote }
    }

    // Replace bits in target with bits captured from variables outside the macro.
    pub fn replace(&self, target: &Expr) -> TokenStream {
        let t = self.width.to_token_stream();
        let mut mask = 0u128;
        let mut replacements = Vec::new();
        for (name, locations) in &self.locations_by_name {
            assert_eq!(locations.len(), 1);
            let name = name.to_ident();
            let shift = locations[0].mask_offset();
            let field = quote! { #t::try_from(#name).unwrap() << #shift };
            replacements.push(field);
            mask |= locations[0].to_mask();
        }

        let mut literal_quote = quote! {};
        if let Some(literal) = self.characters.extract_literal() {
            mask |= self.characters.literal_mask();
            let t = self.width.to_token_stream();
            literal_quote = quote! { | (#literal as #t) };
        }

        let mask = !mask;
        quote! { (#target & #mask as #t) | (#(#replacements)|*) #literal_quote }
    }


    // Substitute fields into template (not macro arguments nor captured from context).
    // WRONG ASSUMPTIONS:
    // * Each name only has a single segment.
    pub fn substitute_fields(&self, fields: Vec<Field>) -> TokenStream {
        let fields: BTreeMap<Name, Field> = fields.into_iter()
            .map(|field| (field.name(), field))
            .collect();
        let mut field_streams = Vec::new();
        for (name, locations) in &self.locations_by_name {
            assert_eq!(locations.len(), 1);
            let location = locations[0];
            let field = fields[name].clone()
                .shift_left(location.mask_offset())
                .widen(self.width);
            assert_eq!(location.len(), field.len());
            field_streams.push(field.to_token_stream());
        }

        let mut literal_quote = quote! {};
        if let Some(literal) = self.characters.extract_literal() {
            let t = self.width.to_token_stream();
            literal_quote = quote! { | (#literal as #t) };
        }

        quote! { (#(#field_streams)|*) #literal_quote }
    }

    // Convert a template expression into a String. Useful for error messages.
    pub fn template_string(template: &Expr) -> String {
        let Expr::Lit(template) = template.clone() else { panic!() };
        let Lit::Str(template) = template.lit else { panic!() };
        template.value()
    }

    // True if any placeholders (periods) are present. Used in APIs that don't accept placeholders.
    pub fn has_placeholders(&self) -> bool {
        self.characters.has_placeholders()
    }

    // Convert the template into a uniquely-identifying struct name.
    pub fn to_struct_name(&self) -> Ident {
        let struct_name_suffix: String = self.characters.to_string()
            // Underscores work in struct names, periods do not.
            .replace('.', "_");
        format_ident!("{}", format!("Fields·{}", struct_name_suffix))
    }
}

// TODO: Reject base 64 special characters.
fn reject_higher_base_chars(text: &str, base: Base) {
    let banned_chars: BTreeSet<char> = match base {
        Base::Binary => ('2'..='9').chain('A'..='Z').collect(),
        Base::Hexadecimal => ('G'..='Z').collect(),
    };

    let chars: BTreeSet<char> = text.chars().collect();
    let rejections: Vec<char> = chars.intersection(&banned_chars).copied().collect();
    assert!(rejections.is_empty(),
        "Invalid characters for base {} detected: {rejections:?}. Did you mean to use a higher base?",
        base as u8,
    );
}
