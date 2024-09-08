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
        reject_higher_base_chars(&template_string, base);
        let characters = Characters::from_str(&template_string, base);

        let len: u8 = characters.len();
        let input_type = Type::for_template(len);
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

    pub fn extract_fields(&self, input: &Expr, min_size: Option<Type>) -> Vec<Field> {
        self.locations_by_name.iter()
            .map(|(name, locations)| Field::new(*name, self.input_type, input, self.precision, min_size, locations))
            .collect()
    }

    pub fn combine_variables(&self, on_overflow: OnOverflow) -> TokenStream {
        let t = self.input_type.to_token_stream();
        let mut field_streams = Vec::new();
        for (name, locations) in &self.locations_by_name {
            assert_eq!(locations.len(), 1);
            let var = name.to_char();
            let name = name.to_ident();
            let shift = locations[0].mask_offset();
            let mask = locations[0].to_unshifted_mask();
            let len = locations[0].len();
            let field = match on_overflow {
                OnOverflow::Corrupt  => quote! { #t::from(#name) << #shift },
                OnOverflow::Wrap     => quote! { (#t::from(#name) & (#mask as #t)) << #shift },
                OnOverflow::Panic    => quote! {
                    {
                        let n = #t::from(#name);
                        assert!(n <= #mask as #t,
                            "Variable {} is too big for its location in the template. {n} > {} ({} bits)", #var, #mask, #len);
                        n << #shift
                    }
                },
                OnOverflow::Saturate => quote! {
                    {
                        let mut n = #t::from(#name);
                        let mask = #mask as #t;
                        if n > mask {
                            n = mask;
                        }
                        n << #shift
                    }
                },
            };
            field_streams.push(field);
        }

        let mut literal_quote = quote! {};
        if let Some(literal) = self.characters.extract_literal() {
            let t = self.input_type.to_token_stream();
            literal_quote = quote! { | (#literal as #t) };
        }

        quote! { (#(#field_streams)|*) #literal_quote }
    }

    pub fn combine_with(&self, exprs: &[Expr]) -> TokenStream {
        let t = self.input_type.to_token_stream();
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
            let t = self.input_type.to_token_stream();
            literal_quote = quote! { | (#literal as #t) };
        }

        quote! { (#(#field_streams)|*) #literal_quote }
    }

    pub fn replace(&self, target: &Expr) -> TokenStream {
        let t = self.input_type.to_token_stream();
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
            let t = self.input_type.to_token_stream();
            literal_quote = quote! { | (#literal as #t) };
        }

        let mask = !mask;
        quote! { (#target & #mask as #t) | (#(#replacements)|*) #literal_quote }
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

#[derive(Debug, Clone, Copy)]
pub enum OnOverflow {
    Wrap,
    Panic,
    Corrupt,
    Saturate,
}
