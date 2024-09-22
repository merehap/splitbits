use proc_macro2::Ident;
use quote::format_ident;

// A single char Field name.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
pub struct Name(char);

impl Name {
    // Create a new name, failing if a non-ascii-lowercase char is provided.
    pub fn new(raw_name: char) -> Result<Self, String> {
        if raw_name.is_ascii_lowercase() {
            Ok(Self(raw_name))
        } else {
            Err(format!("'{raw_name}' is not a valid Name."))
        }
    }

    // Convert to a unicode char.
    pub const fn to_char(self) -> char {
        self.0
    }

    // Convert for use in macro output.
    pub fn to_ident(self) -> Ident {
        format_ident!("{}", self.to_char())
    }
}
