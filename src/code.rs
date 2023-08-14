use std::fmt::Debug;

/// A Turing Machine code, represented by its digits.
/// 
/// # Example
/// ```rust
/// use turing_machine_ai::code::Code;
/// 
/// let code = Code::from_digits(5, 4, 3);
/// assert_eq!(code.triangle(), 5);
/// assert_eq!(code.square(), 4);
/// assert_eq!(code.circle(), 3);
/// ```
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Code {
    triangle: u8,
    square: u8,
    circle: u8,
}

impl Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Code {
            triangle,
            circle,
            square,
        } = self;
        write!(f, "△: {triangle}, □: {square}, ○: {circle}")
    }
}

impl Code {
    pub fn triangle(self) -> u8 {
        self.triangle
    }

    pub fn square(self) -> u8 {
        self.square
    }

    pub fn circle(self) -> u8 {
        self.circle
    }

    /// Count the appearances of a particular digit.
    /// 
    /// # Example
    /// ```rust
    /// use turing_machine_ai::code::Code;
    /// 
    /// assert_eq!(Code::from_digits(2, 3, 4).count_digit(2), 1);
    /// assert_eq!(Code::from_digits(2, 3, 2).count_digit(2), 2);
    /// ```
    pub fn count_digit(&self, digit: u8) -> usize {
        (if self.triangle == digit { 1usize } else { 0 })
            + (if self.square == digit { 1 } else { 0 })
            + (if self.circle == digit { 1 } else { 0 })
    }

    /// Count the even digits.
    /// 
    /// # Example
    /// ```
    /// use turing_machine_ai::code::Code;
    /// 
    /// assert_eq!(Code::from_digits(2, 3, 4).count_even(), 2);
    /// ```
    pub fn count_even(&self) -> usize {
        (if self.triangle % 2 == 0 { 1 } else { 0 })
            + (if self.square % 2 == 0 { 1 } else { 0 })
            + (if self.circle % 2 == 0 { 1 } else { 0 })
    }

    pub fn from_digits(triangle: u8, square: u8, circle: u8) -> Code {
        assert!((1..=5).contains(&triangle));
        assert!((1..=5).contains(&square));
        assert!((1..=5).contains(&circle));
        Code {
            triangle,
            square,
            circle,
        }
    }

    fn to_index(self) -> usize {
        (usize::from(self.triangle) - 1)
            + (usize::from(self.square) - 1) * 5
            + (usize::from(self.circle) - 1) * 25
    }

    fn from_index(mut index: usize) -> Self {
        let triangle = (index % 5) as u8 + 1;
        index /= 5;
        let square = (index % 5) as u8 + 1;
        index /= 5;
        let circle = (index % 5) as u8 + 1;
        Code {
            triangle,
            square,
            circle,
        }
    }
}

/// A set of possible codes.
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
    /// Add a code into this code set.
    pub fn insert(&mut self, code: Code) {
        self.code_bitmap |= CodeSet::from_single_code(code).code_bitmap;
    }

    pub fn intersected_with(self, code_set: CodeSet) -> CodeSet {
        CodeSet {
            code_bitmap: self.code_bitmap & code_set.code_bitmap,
        }
    }

    pub fn union_with(self, code_set: CodeSet) -> CodeSet {
        CodeSet { code_bitmap: self.code_bitmap | code_set.code_bitmap }
    }

    pub fn from_single_code(code: Code) -> CodeSet {
        CodeSet {
            code_bitmap: 1 << code.to_index(),
        }
    }

    pub fn empty() -> CodeSet {
        CodeSet { code_bitmap: 0 }
    }

    pub fn union(&mut self, other: CodeSet) {
        self.code_bitmap |= other.code_bitmap;
    }

    pub fn all() -> CodeSet {
        let mut code_set = CodeSet::empty();
        for triangle in 1..=5 {
            for square in 1..=5 {
                for circle in 1..=5 {
                    code_set.union(CodeSet::from_single_code(Code::from_digits(
                        triangle, square, circle,
                    )));
                }
            }
        }
        code_set
    }

    /// Get the size of this code set.
    /// 
    /// # Example
    /// ```
    /// use turing_machine_ai::code::CodeSet;
    /// assert_eq!(CodeSet::all().size(), 125);
    /// assert_eq!(CodeSet::empty().size(), 0);
    /// ```
    pub fn size(&self) -> u32 {
        self.code_bitmap.count_ones()
    }

    pub fn from_closure(checker: fn(Code) -> bool) -> Self {
        let code_bitmap = (0..125)
            .map(|code_index| {
                let code = Code::from_index(code_index);
                if checker(code) {
                    1 << code_index
                } else {
                    0
                }
            })
            .fold(0, |acc, new| acc | new);
        CodeSet { code_bitmap }
    }

    pub fn contains(&self, code: Code) -> bool {
        self.code_bitmap & (1 << code.to_index()) != 0
    }

    pub fn iter(&self) -> impl Iterator<Item = Code> + '_ {
        (0..125)
            .map(Code::from_index)
            .filter(|code| self.contains(*code))
    }
}

impl FromIterator<Code> for CodeSet {
    fn from_iter<T: IntoIterator<Item = Code>>(iter: T) -> Self {
        let mut code_set = CodeSet::empty();
        for code in iter {
            code_set.insert(code);
        }
        code_set
    }
}

#[cfg(test)]
mod tests {
    use super::{Code, CodeSet};

    #[test]
    fn test_code_set() {
        let code_set = CodeSet::from_closure(|code| code.triangle == 1);
        assert!(code_set.contains(Code::from_digits(1, 2, 3)));
        assert!(!code_set.contains(Code::from_digits(3, 2, 1)));
    }
}
