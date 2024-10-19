use std::collections::{BTreeSet, BTreeMap, VecDeque};

use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident, ToTokens};
use syn::{Expr, Lit};

use crate::base::Base;
use crate::character::Characters;
use crate::field::Field;
use crate::location::Location;
use crate::name::Name;
use crate::location::OnOverflow;
use crate::r#type::{Type, Precision};

/* A sequence of characters used to match and extract bit fields from an integer,
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
    // The locations for a name are ordered from right-to-left (offsets in ascending order).
    locations_by_name: Vec<(Name, Vec<Location>)>,
}

impl Template {
    // Create a Template from a String-format macro expression.
    pub fn from_expr(expr: &Expr, base: Base, precision: Precision) -> Self {
        let template_string = Self::template_string(expr);
        reject_higher_base_chars(&template_string, base);
        let characters = Characters::from_str(&template_string, base);

        let name_offsets: VecDeque<(u8, Option<Name>)> = characters.iter()
            .rev()
            .enumerate()
            .map(|(offset, character)| (u8::try_from(offset).unwrap(), character.to_name()))
            .collect();

        let width = Type::for_template(characters.width())
            .expect("Template must have a valid width");
        let mut locations_by_name: Vec<(Name, Vec<Location>)> = Vec::new();
        for name in characters.to_names() {
            let mut name_offsets = name_offsets.clone();
            let mut locations = Vec::new();
            while let Some(&(mask_offset, _)) = name_offsets.front() {
                let mut width = 0;
                while name_offsets.pop_front().map(|(_, n)| n) == Some(Some(name)) {
                    width += 1;
                }

                if width > 0 {
                    locations.push(Location { width, mask_offset });
                }
            }

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
    pub fn combine_with_context(&self, on_overflow: OnOverflow) -> TokenStream {
        let mut field_streams = Vec::new();
        for (name, locations) in &self.locations_by_name {
            let mut streams = self.create_field_streams(
                *name, Box::new(name.to_ident()), &locations, on_overflow);
            field_streams.append(&mut streams);
        }

        self.combine_with_literal(&field_streams)
    }

    // Substitute macro arguments into the template.
    pub fn combine_with_args(&self, on_overflow: OnOverflow, exprs: &[Expr]) -> TokenStream {
        for expr in exprs {
            if let Expr::Lit(template) = expr.clone() {
                if let Lit::Str(template) = template.lit {
                    panic!("Only one template must be present, \
                        but found this string literal too: '{}'.", template.value());
                };
            };
        }

        assert_eq!(exprs.len(), self.locations_by_name.len(),
            "The number of inputs must be equal to the number of names in the template.",
        );

        let mut field_streams = Vec::new();
        for ((name, locations), expr) in self.locations_by_name.iter().zip(exprs.iter()) {
            let mut streams = self.create_field_streams(
                *name, Box::new(expr.clone()), &locations, on_overflow);
            field_streams.append(&mut streams);
        }

        self.combine_with_literal(&field_streams)
    }

    // Replace bits in target with bits captured from variables outside the macro.
    pub fn replace(&self, on_overflow: OnOverflow, target: &Expr) -> TokenStream {
        let t = self.width.to_token_stream();
        // The mask allows us to clear to relevant bits in the target before applying replacements.
        let mut replacement_mask = 0u128;
        let mut replacements = Vec::new();
        for (name, locations) in &self.locations_by_name {
            let mut segment_offset = 0;
            for i in 0..locations.len() {
                let location = locations[i];
                let var = name;
                let name = name.to_ident();
                let mask = location.to_unshifted_mask();
                let width = self.width.to_token_stream();
                let shift = if segment_offset == 0 {
                    quote! {}
                } else {
                    quote! { >> #segment_offset }
                };

                // TODO: Removing this mask may be a breaking change. Figure out why
                // the test split_then_combine_into_fragments fails when the mask is removed for
                // it, then add a similar test for replacebits.
                let mask = if i == locations.len() - 1 {
                    quote! {}
                } else {
                    quote! { & (#mask as #width) }
                };

                // TODO: Should this be an expect() instead of unwrap()?
                let segment = quote! { ((#width::try_from(#name #shift).unwrap()) #mask) };
                segment_offset += location.width();
                let field = location.place_field_segment(
                    var.to_token_stream(),
                    segment,
                    self.width,
                    on_overflow,
                );
                replacements.push(field);
                replacement_mask |= location.to_mask();
            }
        }

        let mut literal_quote = quote! {};
        if let Some(literal) = self.characters.extract_literal() {
            replacement_mask |= self.characters.literal_mask();
            let t = self.width.to_token_stream();
            literal_quote = quote! { | (#literal as #t) };
        }

        let replacement_mask = !replacement_mask;
        quote! { (#target & #replacement_mask as #t) | (#(#replacements)|*) #literal_quote }
    }

    // Substitute Fields into template (not macro arguments nor captured from context).
    pub fn substitute_fields(&self, fields: Vec<Field>) -> TokenStream {
        let fields: BTreeMap<Name, Field> = fields.into_iter()
            .map(|field| (field.name(), field))
            .collect();
        let mut field_streams = Vec::new();
        for (name, locations) in &self.locations_by_name {
            let field = fields[name].clone();
            let n = name.to_ident();
            assert_eq!(field.width(), locations.iter().map(|l| l.width).sum(),
                "The width of field '{n}' must match between the input templates and the output template.");
            let field = field.widen(self.width);
            let segment = Box::new(field.to_token_stream());
            // Input and output fields having unequal lengths should fail at compile time,
            // so go with OnOverflow::Corrupt since it is the most efficient option.
            let mut streams = self.create_field_streams(*name, segment, locations, OnOverflow::Corrupt);
            field_streams.append(&mut streams);
        }

        self.combine_with_literal(&field_streams)
    }

    // Convert a template expression into a String. Useful for error messages.
    pub fn template_string(template: &Expr) -> String {
        let template_text = quote! { #template };
        let Expr::Lit(template) = template.clone() else {
            panic!("The template expression must come after the input value(s), \
                and must be a literal, but found:\n{template_text}");
        };
        let Lit::Str(template) = template.lit else {
            panic!("The template expression must come after the input value(s), \
                and must be a string literal, but found:\n{template_text}");
        };
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

    fn create_field_streams(
        &self,
        name: Name,
        var: Box<dyn ToTokens>,
        locations: &[Location],
        on_overflow: OnOverflow,
    ) -> Vec<TokenStream> {
        let mut field_streams = Vec::new();

        let mut segment_offset = 0;
        for i in 0..locations.len() {
            let location = locations[i];
            let mask = location.to_unshifted_mask();
            let width = self.width.to_token_stream();
            let shift = if segment_offset == 0 {
                quote! {}
            } else {
                quote! { >> #segment_offset }
            };

            let mask = if i == locations.len() - 1 {
                quote! {}
            } else {
                quote! { & (#mask as #width) }
            };

            let segment = quote! { ((#width::from(#var #shift)) #mask) };
            segment_offset += location.width();
            let field_stream = location.place_field_segment(
                name.to_token_stream(),
                segment,
                self.width,
                on_overflow,
            );
            field_streams.push(field_stream);
        }

        field_streams
    }

    fn combine_with_literal(&self, field_streams: &[TokenStream]) -> TokenStream {
        if let Some(literal) = self.characters.extract_literal() {
            let width = self.width.to_token_stream();
            quote! { (#(#field_streams)|*) | (#literal as #width) }
        } else {
            quote! { #(#field_streams)|* }
        }
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
