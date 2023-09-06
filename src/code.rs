//! This module contains functionality for working with codes and sets of codes.

use std::fmt::Debug;
use std::num::NonZeroU128;

use thiserror::Error;

/// A Turing Machine code, represented by a flipped bit in a [`u128`]. This is
/// the most efficient format for use with [`Set`] since it allows for fast
/// set inclusion checks.
#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct Code {
    bits: NonZeroU128,
}

/// This error may be returned when attempting to construct an invalid code.
#[derive(Copy, Clone, Eq, PartialEq, Debug, Error, Hash)]
pub enum Error {
    /// Returned when attempting to construct an invalid code.
    #[error("the provided digits do not form a valid code")]
    InvalidDigits,
}

/// Returned by [`Code::is_ascending_or_descending`] to indicate whether the code
/// has an ascending or descending sequence.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Hash)]
pub enum Order {
    /// The code has an ascending sequence, i.e. `(1, 2, 3)`.
    Ascending,
    /// The code has a descending sequence, i.e. `(4, 3, 2)`.
    Descending,
    /// The code has neither an ascending or descending sequence.
    NoOrder,
}

impl Code {
    fn digits_to_index(triangle: u8, square: u8, circle: u8) -> usize {
        (usize::from(triangle) - 1) + (usize::from(square) - 1) * 5 + (usize::from(circle) - 1) * 25
    }

    /// Get the code with the given digits.
    ///
    /// # Errors
    /// If the provided digits do not lie in the range `1..=5`, the code is
    /// invalid and the error [`Error::InvalidDigits`] will be returned.
    ///
    /// # Examples
    /// ```rust
    /// use turing_machine_ai::code;
    /// assert!(code::Code::from_digits(1, 2, 3).is_ok());
    /// assert_eq!(code::Code::from_digits(3, 4, 9), Err(code::Error::InvalidDigits));
    /// ```
    // We conclude that this function cannot actually panic, but test this as
    // well through the proptest in the `::tests` module.
    #[allow(clippy::missing_panics_doc)]
    pub fn from_digits(triangle: u8, square: u8, circle: u8) -> Result<Self, Error> {
        if !(1..=5).contains(&triangle) || !(1..=5).contains(&square) || !(1..=5).contains(&circle)
        {
            Err(Error::InvalidDigits)
        } else {
            Ok(Code {
                bits: (1 << Self::digits_to_index(triangle, square, circle))
                    .try_into()
                    // We have checked that the index is between 0-124 incl.
                    // and so this can never fail.
                    .unwrap(),
            })
        }
    }

    /// Get the digits of this code.
    /// ```rust
    /// use turing_machine_ai::code::Code;
    ///
    /// let code = Code::from_digits(5, 4, 3)?;
    /// assert_eq!(code.digits(), (5, 4, 3));
    ///
    /// let code_2 = Code::from_digits(1, 3, 4)?;
    /// assert_eq!(code_2.digits(), (1, 3, 4));
    /// # Ok::<(), turing_machine_ai::code::Error>(())
    /// ```
    #[must_use]
    // It is not possible to make this function panic, since all digits will
    // lie between 1-5. We test this throguh property testing.
    #[allow(clippy::missing_panics_doc)]
    pub fn digits(self) -> (u8, u8, u8) {
        let index = self.bits.trailing_zeros();
        let triangle = (index % 5) + 1;
        let square = ((index / 5) % 5) + 1;
        let circle = ((index / 25) % 5) + 1;
        (
            u8::try_from(triangle).unwrap(),
            u8::try_from(square).unwrap(),
            u8::try_from(circle).unwrap(),
        )
    }

    /// Returns the digit for the triangle symbol in this code.
    #[must_use]
    pub fn triangle(self) -> u8 {
        self.digits().0
    }

    /// Returns the digit for the square symbol in this code.
    #[must_use]
    pub fn square(self) -> u8 {
        self.digits().1
    }

    /// Returns the digit for the circle symbol in this code.
    #[must_use]
    pub fn circle(self) -> u8 {
        self.digits().2
    }

    /// Get the sum of the digits.
    ///
    /// ```rust
    /// use turing_machine_ai::code::Code;
    /// let code = Code::from_digits(5, 2, 4)?;
    /// assert_eq!(code.digit_sum(), 11);
    /// # Ok::<(), turing_machine_ai::code::Error>(())
    /// ```
    #[must_use]
    pub fn digit_sum(self) -> u8 {
        let (a, b, c) = self.digits();
        a + b + c
    }

    /// Count the appearances of a particular digit.
    ///
    /// # Example
    /// ```rust
    ///
    /// use turing_machine_ai::code::Code;
    ///
    /// assert_eq!(Code::from_digits(2, 3, 4)?.count_digit(2), 1);
    /// assert_eq!(Code::from_digits(2, 3, 2)?.count_digit(2), 2);
    /// # Ok::<(), turing_machine_ai::code::Error>(())
    /// ```
    #[must_use]
    pub fn count_digit(&self, digit: u8) -> usize {
        usize::from(self.triangle() == digit)
            + usize::from(self.square() == digit)
            + usize::from(self.circle() == digit)
    }

    /// Count the even digits.
    ///
    /// # Example
    /// ```
    /// use turing_machine_ai::code::Code;
    ///
    /// assert_eq!(Code::from_digits(2, 3, 4)?.count_even(), 2);
    /// # Ok::<(), turing_machine_ai::code::Error>(())
    /// ```
    #[must_use]
    pub fn count_even(&self) -> usize {
        usize::from(self.triangle() % 2 == 0)
            + usize::from(self.square() % 2 == 0)
            + usize::from(self.circle() % 2 == 0)
    }

    /// Number of digits in ascending or descending order as specified by
    /// verifier 25.
    #[must_use]
    pub fn sequence_ascending_or_descending(&self) -> usize {
        let (t, s, c) = self.digits();
        if (t + 1 == s && s + 1 == c) || (t == s + 1 && s == c + 1) {
            3
        } else if t + 1 == s || s + 1 == c || t == s + 1 || s == c + 1 {
            2
        } else {
            0
        }
    }

    /// Number of digits in ascending order.
    /// ```
    /// use turing_machine_ai::code::Code;
    /// assert_eq!(Code::from_digits(2, 3, 4)?.sequence_ascending(), 3);
    /// assert_eq!(Code::from_digits(2, 3, 3)?.sequence_ascending(), 2);
    /// assert_eq!(Code::from_digits(1, 3, 5)?.sequence_ascending(), 0);
    /// # Ok::<(), turing_machine_ai::code::Error>(())
    /// ```
    #[must_use]
    pub fn sequence_ascending(self) -> usize {
        let (t, s, c) = self.digits();
        if t + 1 == s && s + 1 == c {
            3
        } else if t + 1 == s || s + 1 == c {
            2
        } else {
            0
        }
    }

    /// Counts the repetitions of the most frequent number, à la verifier card
    /// 20.
    ///
    /// ```rust
    /// use turing_machine_ai::code::Code;
    /// assert_eq!(Code::from_digits(2, 2, 2)?.repeating_numbers(), 2);
    /// assert_eq!(Code::from_digits(1, 1, 2)?.repeating_numbers(), 1);
    /// assert_eq!(Code::from_digits(1, 2, 1)?.repeating_numbers(), 1);
    /// assert_eq!(Code::from_digits(1, 2, 5)?.repeating_numbers(), 0);
    /// # Ok::<(), turing_machine_ai::code::Error>(())
    /// ```
    #[must_use]
    pub fn repeating_numbers(self) -> usize {
        match self.digits() {
            (a, b, c) if a == b && b == c => 2,
            (a, b, c) if a == b || b == c || a == c => 1,
            _ => 0,
        }
    }

    /// Provides the order of the digits as in verifier 22.
    ///
    /// ```rust
    /// use turing_machine_ai::code::{Code, Order};
    /// assert_eq!(Code::from_digits(1, 3, 5)?.is_ascending_or_descending(), Order::Ascending);
    /// assert_eq!(Code::from_digits(4, 2, 1)?.is_ascending_or_descending(), Order::Descending);
    /// assert_eq!(Code::from_digits(2, 3, 1)?.is_ascending_or_descending(), Order::NoOrder);
    /// # Ok::<(), turing_machine_ai::code::Error>(())
    /// ```
    #[must_use]
    pub fn is_ascending_or_descending(self) -> Order {
        let (triangle, square, circle) = self.digits();
        if triangle < square && square < circle {
            Order::Ascending
        } else if triangle > square && square > circle {
            Order::Descending
        } else {
            Order::NoOrder
        }
    }
}

impl Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (triangle, circle, square) = self.digits();
        write!(f, "△: {triangle}, □: {square}, ○: {circle}")
    }
}

/// A struct representing a set of codes.
#[derive(Copy, Clone, Eq, PartialEq, Hash, Default)]
pub struct Set {
    code_bitmap: u128,
}

impl Debug for Set {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "△ □ ○")?;
        for code in self.into_iter() {
            writeln!(f, "{} {} {}", code.triangle(), code.square(), code.circle())?;
        }
        Ok(())
    }
}

impl Set {
    /// Create a new code set, containing only the provided code. This is a
    /// free operation.
    #[must_use]
    pub fn new_from_code(code: Code) -> Self {
        Set {
            code_bitmap: code.bits.get(),
        }
    }

    /// Insert the given code into this code set.
    ///
    /// # Examples
    /// ```rust
    /// use turing_machine_ai::code::{Code, Set};
    ///
    /// let mut set = Set::empty();
    /// let code = Code::from_digits(1, 2, 3)?;
    /// set.insert(code);
    /// assert!(set.contains(code));
    /// # Ok::<(), turing_machine_ai::code::Error>(())
    /// ```
    pub fn insert(&mut self, code: Code) {
        self.code_bitmap |= Set::new_from_code(code).code_bitmap;
    }

    /// Get a new code set, that contains only the elements contained in both
    /// this set, as well as the provided set.
    #[must_use]
    pub fn intersected_with(self, code_set: Set) -> Set {
        Set {
            code_bitmap: self.code_bitmap & code_set.code_bitmap,
        }
    }

    /// Get a new code set that contains all elements contained in either this
    /// set, or the provided set.
    #[must_use]
    pub fn union_with(self, code_set: Set) -> Set {
        Set {
            code_bitmap: self.code_bitmap | code_set.code_bitmap,
        }
    }

    /// Return an empty set.
    ///
    /// # Example
    /// ```rust
    /// use turing_machine_ai::code::Set;
    ///
    /// let empty_set = Set::empty();
    /// assert_eq!(empty_set.size(), 0);
    /// ```
    #[must_use]
    pub fn empty() -> Set {
        Set { code_bitmap: 0 }
    }

    /// Return a set containing all codes.
    ///
    /// # Example
    /// ```rust
    /// use turing_machine_ai::code::Set;
    ///
    /// let complete_set = Set::all();
    /// assert_eq!(complete_set.size(), 125);
    /// ```
    #[must_use]
    pub fn all() -> Set {
        Set {
            code_bitmap: (1 << 125) - 1,
        }
    }

    /// Get the size of this code set.
    ///
    /// # Example
    /// ```
    /// use turing_machine_ai::code::Set;
    /// assert_eq!(Set::all().size(), 125);
    /// assert_eq!(Set::empty().size(), 0);
    /// ```
    #[must_use]
    pub fn size(self) -> u32 {
        self.code_bitmap.count_ones()
    }

    /// Construct a new code set based on a closure that returns `true` for any
    /// code that must be in the set.
    pub fn from_closure(checker: fn(Code) -> bool) -> Self {
        Set::all()
            .into_iter()
            .filter(|code| checker(*code))
            .collect()
    }

    /// Returns whether the given code is part of this set.
    /// ```rust
    /// use turing_machine_ai::code::{Set, Code};
    /// let code_1 = Code::from_digits(1, 2, 3)?;
    /// let code_2 = Code::from_digits(3, 3, 5)?;
    /// let set = Set::new_from_code(code_1);
    /// assert!(set.contains(code_1));
    /// assert!(!set.contains(code_2));
    /// # Ok::<(), turing_machine_ai::code::Error>(())
    /// ```
    #[must_use]
    pub fn contains(self, code: Code) -> bool {
        (self.code_bitmap & code.bits.get()) != 0
    }
}

impl IntoIterator for Set {
    type IntoIter = SetIterator;
    type Item = Code;
    fn into_iter(self) -> Self::IntoIter {
        SetIterator {
            set: self,
            current: 1,
        }
    }
}

/// The iterator for a set.
pub struct SetIterator {
    set: Set,
    current: u128,
}

impl Iterator for SetIterator {
    type Item = Code;
    fn next(&mut self) -> Option<Self::Item> {
        while self.current < (1 << 125) {
            let code = Code {
                bits: self.current.try_into().unwrap(),
            };
            self.current <<= 1;
            if self.set.contains(code) {
                return Some(code);
            }
        }
        None
    }
}

impl FromIterator<Code> for Set {
    /// Create a new code set containing all codes in the iterator.
    fn from_iter<T: IntoIterator<Item = Code>>(iter: T) -> Self {
        let mut code_set = Set::empty();
        for code in iter {
            code_set.insert(code);
        }
        code_set
    }
}

impl FromIterator<Set> for Set {
    fn from_iter<T: IntoIterator<Item = Set>>(iter: T) -> Self {
        let mut code_set = Set::empty();
        for new_code_set in iter {
            code_set = code_set.union_with(new_code_set);
        }
        code_set
    }
}

#[cfg(test)]
mod tests {
    use super::{Code, Set};

    #[test]
    fn test_code_set() {
        let code_set = Set::from_closure(|code| code.triangle() == 1);
        assert!(code_set.contains(Code::from_digits(1, 2, 3).unwrap()));
        assert!(!code_set.contains(Code::from_digits(3, 2, 1).unwrap()));
    }

    use proptest::prelude::*;

    proptest! {
        // We unwrap because there can be no panic, but use testing for more
        // confidence.
        #[test]
        fn test_no_panic_from_digits(triangle in 0..u8::MAX,
            square in 0..u8::MAX,
            circle in 0..u8::MAX) {
            let _ = Code::from_digits(triangle, square, circle);
        }
    }

    proptest! {
        // Test that obtaining the digits can never panic, and that the result is correct.
        #[test]
        fn test_no_panic_digits(triangle in 1..=5u8,
            square in 1..=5u8,
            circle in 1..=5u8
        ) {
            let code = Code::from_digits(triangle, square, circle)?;
            assert_eq!(code.digits(), (triangle, square, circle));
        }
    }
}
