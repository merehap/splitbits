use proc_macro2::Ident;
use quote::format_ident;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Name(char);

impl Name {
    pub fn new(raw_name: char) -> Result<Self, String> {
        if raw_name.is_ascii_lowercase() {
            Ok(Self(raw_name))
        } else {
            Err(format!("'{raw_name}' is not a valid Name."))
        }
    }

    pub const fn to_char(self) -> char {
        self.0
    }

    pub fn to_ident(self) -> Ident {
        format_ident!("{}", self.to_char())
    }
}
