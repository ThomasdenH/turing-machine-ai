use std::num::NonZeroU128;
use std::fmt::Debug;

use thiserror::Error;

/// A Turing Machine code, represented by a flipped bit in a `u128`. This is
/// the most efficient format for use with `CodeSet` since it allows for fast
/// set inclusion checks.
#[derive(Eq, PartialEq, Copy, Clone)]
pub struct Code {
    bits: NonZeroU128,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Error)]
pub enum CodeError {
    #[error("the provided digits do not form a valid code")]
    InvalidDigits
}

impl Code {
    fn digits_to_index(triangle: u8, square: u8, circle: u8) -> usize {
        (usize::from(triangle) - 1)
            + (usize::from(square) - 1) * 5
            + (usize::from(circle) - 1) * 25
    }

    fn from_index(index: usize) -> Result<Self, CodeError> {
        if index < 125 {
            Ok(Code { bits: (1 << index).try_into().unwrap() })
        } else {
            Err(CodeError::InvalidDigits)
        }
    }

    pub fn from_digits(triangle: u8, square: u8, circle: u8) -> Result<Self, CodeError> {
        if !(1..=5).contains(&triangle) || !(1..=5).contains(&square) || !(1..=5).contains(&circle) {
            Err(CodeError::InvalidDigits)
        } else {
            Ok(Code {
                bits: (1 << Self::digits_to_index(triangle, square, circle)).try_into().unwrap()
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
    /// # Ok::<(), turing_machine_ai::code::CodeError>(())
    /// ```
    pub fn digits(self) -> (u8, u8, u8) {
        let index = self.bits.trailing_zeros();
        let triangle = (index % 5) + 1;
        let square = ((index / 5) % 5) + 1;
        let circle = ((index / 25) % 5) + 1;
        (triangle as u8, square as u8, circle as u8)
    }

    pub fn triangle(self) -> u8 {
        self.digits().0
    }

    pub fn square(self) -> u8 {
        self.digits().1
    }

    pub fn circle(self) -> u8 {
        self.digits().2
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
    /// # Ok::<(), turing_machine_ai::code::CodeError>(())
    /// ```
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
    /// # Ok::<(), turing_machine_ai::code::CodeError>(())
    /// ```
    pub fn count_even(&self) -> usize {
        usize::from(self.triangle() % 2 == 0)
            + usize::from(self.square() % 2 == 0)
            + usize::from(self.circle() % 2 == 0)
    }

    /// Number of digits in ascending or descending order as specified by
    /// verifier 25.
    pub fn numbers_ascending_or_descending(&self) -> usize {
        match (self.triangle() as i8 - self.square() as i8, self.square() as i8 - self.circle() as i8) {
            (1, 1) | (-1, -1) => 3,
            (1, _) | (_, 1) | (-1, _) | (_, -1) => 2,
            _ => 0
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
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct CodeSet {
    code_bitmap: u128,
}

impl Debug for CodeSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "△ □ ○")?;
        for code in self.iter() {
            writeln!(f, "{} {} {}", code.triangle(), code.square(), code.circle())?;
        }
        Ok(())
    }
}

impl CodeSet {
    /// Create a new code set, containing only the provided code. This is a
    /// free operation.
    pub fn new_from_code(code: Code) -> Self {
        CodeSet { code_bitmap: code.bits.get() }
    }

    /// Insert the given code into this code set.
    pub fn insert(&mut self, code: Code) {
        self.code_bitmap |= CodeSet::new_from_code(code).code_bitmap;
    }

    /// Get a new code set, that contains only the elements contained in both
    /// this set, as well as the provided set.
    pub fn intersected_with(self, code_set: CodeSet) -> CodeSet {
        CodeSet {
            code_bitmap: self.code_bitmap & code_set.code_bitmap,
        }
    }

    /// Get a new code set that contains all elements contained in either this
    /// set, or the provided set.
    pub fn union_with(self, code_set: CodeSet) -> CodeSet {
        CodeSet {
            code_bitmap: self.code_bitmap | code_set.code_bitmap,
        }
    }

    /// Return an empty set.
    /// 
    /// # Example
    /// ```rust
    /// use turing_machine_ai::code::CodeSet;
    /// 
    /// let empty_set = CodeSet::empty();
    /// assert_eq!(empty_set.size(), 0);
    /// ```
    pub fn empty() -> CodeSet {
        CodeSet { code_bitmap: 0 }
    }

    /// Return a set containing all codes.
    /// 
    /// # Example
    /// ```rust
    /// use turing_machine_ai::code::CodeSet;
    /// 
    /// let complete_set = CodeSet::all();
    /// assert_eq!(complete_set.size(), 125);
    /// ```
    pub fn all() -> CodeSet {
        CodeSet {
            code_bitmap: (1 << 125) - 1
        }
    }

    /// Get the size of this code set.
    ///
    /// # Example
    /// ```
    /// use turing_machine_ai::code::CodeSet;
    /// assert_eq!(CodeSet::all().size(), 125);
    /// assert_eq!(CodeSet::empty().size(), 0);
    /// ```
    pub fn size(self) -> u32 {
        self.code_bitmap.count_ones()
    }

    pub fn from_closure(checker: fn(Code) -> bool) -> Self {
        CodeSet::all().iter().filter(|code| checker(*code)).collect()
    }

    pub fn contains(self, code: Code) -> bool {
        (self.code_bitmap & code.bits.get()) != 0
    }

    /// Iterate through all codes in this set.
    pub fn iter(self) -> impl Iterator<Item = Code> {
        (0..125).map(Code::from_index)
            .map(Result::unwrap)
            .filter(move |code| self.contains(*code))
    }
}

impl FromIterator<Code> for CodeSet {
    /// Create a new code set containing all codes in the iterator.
    fn from_iter<T: IntoIterator<Item = Code>>(iter: T) -> Self {
        let mut code_set = CodeSet::empty();
        for code in iter {
            code_set.insert(code);
        }
        code_set
    }
}

impl FromIterator<CodeSet> for CodeSet {
    fn from_iter<T: IntoIterator<Item = CodeSet>>(iter: T) -> Self {
        let mut code_set = CodeSet::empty();
        for new_code_set in iter {
            code_set = code_set.union_with(new_code_set);
        }
        code_set
    }
}

#[cfg(test)]
mod tests {
    use super::{Code, CodeSet};

    #[test]
    fn test_code_set() {
        let code_set = CodeSet::from_closure(|code| code.triangle() == 1);
        assert!(code_set.contains(Code::from_digits(1, 2, 3).unwrap()));
        assert!(!code_set.contains(Code::from_digits(3, 2, 1).unwrap()));
    }
}
