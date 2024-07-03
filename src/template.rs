use std::collections::BTreeMap;

use itertools::Itertools;
use proc_macro2::{TokenStream, Ident};
use quote::{quote, format_ident};
use syn::{Expr, Lit};

use crate::base::Base;
use crate::character::{Character, Characters};
use crate::field::Field;
use crate::location::Location;
use crate::name::Name;
use crate::r#type::{Type, Precision};

pub struct Template {
    input_type: Type,
    precision: Precision,
    characters: Characters,
    locations_by_name: Vec<(Name, Vec<Location>)>,
}

impl Template {
    pub fn from_expr(expr: &Expr, base: Base, precision: Precision) -> Template {
        let template_string = Template::template_string(expr);
        let characters = Characters::from_str(&template_string, base);

        let input_type = Type::for_template(characters.len() as u8);
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
                        let len = segment.len() as u8;
                        let mask_offset = segment[0].0 as u8;
                        Some(Location::new(len, mask_offset))
                    } else {
                        None
                    }
                })
                .collect();
            locations_by_name.push((name, locations));
        }

        Template { input_type, precision, characters, locations_by_name }
    }

    pub fn template_string(template: &Expr) -> String {
        let Expr::Lit(template) = template.clone() else { panic!() };
        let Lit::Str(template) = template.lit else { panic!() };
        template.value()
    }

    pub fn has_placeholders(&self) -> bool {
        self.characters.has_placeholders()
    }

    pub fn extract_fields(&self, input: &Expr) -> Vec<Field> {
        self.locations_by_name.iter()
            .map(|(name, locations)| Field::new(*name, self.input_type, input, self.precision, &locations))
            .collect()
    }

    pub fn combine_variables(&self) -> TokenStream {
        let t = self.input_type.to_token_stream();
        let mut field_streams = Vec::new();
        for (name, locations) in &self.locations_by_name {
            assert_eq!(locations.len(), 1);
            let name = name.to_ident();
            let shift = locations[0].mask_offset();
            let field = quote! { #t::from(#name) << #shift };
            field_streams.push(field);
        }

        let mut literal_quote = quote! {};
        if let Some(literal) = self.characters.extract_literal() {
            let t = self.input_type.to_token_stream();
            literal_quote = quote! { | (#literal as #t) };
        }

        quote! { (#(#field_streams)|*) #literal_quote }
    }

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
                .widen(self.input_type);
            assert_eq!(location.len(), field.len());
            field_streams.push(field.to_token_stream());
        }

        let mut literal_quote = quote! {};
        if let Some(literal) = self.characters.extract_literal() {
            let t = self.input_type.to_token_stream();
            literal_quote = quote! { | (#literal as #t) };
        }

        quote! { (#(#field_streams)|*) #literal_quote }
    }

    pub fn to_struct_name(&self) -> Ident {
        let struct_name_suffix: String = self.characters.to_string()
            // Underscores work in struct names, periods do not.
            .replace('.', "_");
        format_ident!("{}", format!("Fields·{}", struct_name_suffix))
    }
}
