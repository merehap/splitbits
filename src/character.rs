use std::fmt;

use itertools::Itertools;

use crate::Base;
use crate::name::Name;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Character {
    Name(Name),
    Placeholder,
    Zero,
    One,
}

impl Character {
    pub fn from_char(c: char) -> Result<Self, String> {
        Ok(match c {
            '.' => Character::Placeholder,
            '0' => Character::Zero,
            '1' => Character::One,
            _   => Character::Name(Name::new(c)?),
        })
    }

    pub fn is_literal(self) -> bool {
        self == Character::Zero || self == Character::One
    }

    pub fn to_name(self) -> Option<Name> {
        if let Character::Name(name) = self {
            Some(name)
        } else {
            None
        }
    }

    pub fn to_char(self) -> char {
        match self {
            Character::Name(name) => name.to_char(),
            Character::Placeholder => '.',
            Character::Zero => '0',
            Character::One  => '1',
        }
    }
}

pub struct Characters(Vec<Character>);

impl Characters {
    // TODO: Make this work with base 16 literals.
    pub fn from_str(text: &str, base: Base) -> Characters {
        let characters: Result<Vec<Character>, String> = text.chars()
            // Spaces are only for human-readability.
            .filter(|&c| c != ' ')
            .map(Character::from_char)
            .collect();

        let characters: Vec<Character> = characters.unwrap().into_iter()
            // Each template char needs to be repeated if we aren't working in base 2.
            .flat_map(|c| std::iter::repeat(c).take(base.bits_per_digit()))
            .collect();

        assert!(characters.len() <= 128);
        Characters(characters)
    }

    pub fn extract_literal(&self) -> Option<u128> {
        if self.0.iter().filter(|c| c.is_literal()).next().is_none() {
            return None;
        }

        let literal_string: String = self.0.iter()
            .map(|&c| if c == Character::One { '1' } else { '0' })
            .collect();
        Some(u128::from_str_radix(&literal_string, 2).unwrap())
    }

    pub fn has_placeholders(&self) -> bool {
        self.0.iter()
            .find(|&&character| character == Character::Placeholder)
            .is_some()
    }

    pub fn to_names(&self) -> Vec<Name> {
        self.0.iter()
            .filter_map(|c| c.to_name())
            .unique()
            .collect()
    }

    pub fn len(&self) -> u8 {
        u8::try_from(self.0.len()).unwrap()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item=&Character> {
        self.0.iter().clone()
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
