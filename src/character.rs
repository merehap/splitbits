use std::collections::BTreeSet;
use std::fmt;

use crate::Base;
use crate::name::Name;

/* A single legal character for a template in its base 2 form.
 * Higher bases must be converted to base 2 to use this.
 */
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Character {
    // A character that is legal within a field name.
    Name(Name),
    // A period '.', indicating that the template will ignore the matching bit.
    Placeholder,
    // A literal '0' bit.
    Zero,
    // A literal '1' bit.
    One,
}

impl Character {
    // Attempt to convert a unicode char to a Character.
    pub fn from_char(c: char) -> Result<Self, String> {
        Ok(match c {
            '.' => Self::Placeholder,
            '0' => Self::Zero,
            '1' => Self::One,
            _   => Self::Name(Name::new(c)?),
        })
    }

    // Whether the character is a literal '0' or '1'.
    pub fn is_literal(self) -> bool {
        self == Self::Zero || self == Self::One
    }

    // Convert the character to a Name, if it is a valid Name.
    pub const fn to_name(self) -> Option<Name> {
        if let Self::Name(name) = self {
            Some(name)
        } else {
            None
        }
    }

    // Convert the character to a unicode char.
    pub const fn to_char(self) -> char {
        match self {
            Self::Placeholder => '.',
            Self::Zero => '0',
            Self::One  => '1',
            Self::Name(name) => name.to_char(),
        }
    }
}

/* Contains consecutive values of type Character.
 * Used as the backing store for a Template.
 * Maximum length is 128.
 */
pub struct Characters(Vec<Character>);

impl Characters {
    /* Given a numeric Base, convert a str to a Characters type.
     * Strips out any spaces as those are for human-reability.
     * Converts non-binary literals into binary literals.
     */
    pub fn from_str(text: &str, base: Base) -> Self {
        let characters: Vec<Character> = text.chars()
            // Spaces are only for human-readability.
            .filter(|&c| c != ' ')
            // Each template char needs to be repeated if we aren't working in base 2.
            .flat_map(|c| {
                let characters: Box<dyn Iterator<Item = Character>>;
                if base == Base::Hexadecimal {
                    if let Some(array) = Self::hex_digit_to_array(c) {
                        characters = Box::new(array.into_iter());
                        return characters;
                    }
                }

                if let Ok(character) = Character::from_char(c) {
                    characters = Box::new(std::iter::repeat(character).take(base.bits_per_digit()));
                } else {
                    panic!("Invalid template char '{c}' in template '{text}'.");
                }

                characters
            })
            .collect();

        assert!(characters.len() <= 128, "Template size was greater than 128 bits. Template: '{text}'");
        Self(characters)
    }

    /* Get the literal that the template corresponds to.
     * Effectively, return the '1's among the Characters, converting everything else to '0's.
     * Return None if there are no literal digits in the template.
     */
    pub fn extract_literal(&self) -> Option<u128> {
        if !self.0.iter().any(|c| c.is_literal()) {
            return None;
        }

        let literal_string: String = self.0.iter()
            .map(|&c| if c == Character::One { '1' } else { '0' })
            .collect();
        Some(u128::from_str_radix(&literal_string, 2).expect("All digits should be '0' or '1'"))
    }

    // Return '1's where there is a literal Character, '0's everywhere else.
    pub fn literal_mask(&self) -> u128 {
        let literal_string: String = self.0.iter()
            .map(|&c| if c == Character::Zero || c == Character::One { '1' } else { '0' })
            .collect();
        u128::from_str_radix(&literal_string, 2).expect("All digits should be '0' or '1'")
    }

    // Return true if there are any periods among the Characters.
    pub fn has_placeholders(&self) -> bool {
        self.0.iter()
            .any(|&character| character == Character::Placeholder)
    }

    // Extract all the unique names that are present in the Characters.
    pub fn to_names(&self) -> Vec<Name> {
        let mut uniques = BTreeSet::new();

        let mut names = Vec::new();
        for character in &self.0 {
            if let Some(name) = character.to_name() && uniques.insert(name) {
                names.push(name);
            }
        }

        names
    }

    // The count of Characters.
    pub fn width(&self) -> u8 {
        u8::try_from(self.0.len()).expect("Template width should be under 256")
    }

    // Iterate from the first to the last Character.
    pub fn iter(&self) -> impl DoubleEndedIterator<Item=&Character> {
        self.0.iter()
    }

    // Hexadecimal digits correspond to 4 (binary) entries of type Character.
    const fn hex_digit_to_array(digit: char) -> Option<[Character; 4]> {
        const fn conv(value: u32) -> Character {
            if value == 0 { Character::Zero } else { Character::One }
        }

        let n = match digit {
            '0'..='9' => digit as u32 - '0' as u32,
            'A'..='F' => digit as u32 - 'A' as u32 + 0xA,
            _ => return None,
        };

        Some([conv(n & 0b1000), conv(n & 0b0100), conv(n & 0b0010), conv(n & 0b0001)])
    }
}

impl fmt::Display for Characters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for &character in &self.0 {
            write!(f, "{}", character.to_char())?;
        }

        Ok(())
    }
}
