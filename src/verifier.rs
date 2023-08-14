use std::fmt::Debug;

use arrayvec::ArrayVec;

use crate::code::{Code, CodeSet};

/// Get a verifier by its (one-indexed) number in the game.
pub fn get_verifier_by_number(number: usize) -> Verifier {
    let mut verifiers: Vec<Option<Verifier>> = vec![None; 48];

    verifiers[0] = Some(Verifier::from_description_and_options(
        "the △ number compared to 1",
        &[
            VerifierOption::from_description_and_closure("△ = 1", |code| code.triangle() == 1),
            VerifierOption::from_description_and_closure("△ > 1", |code| code.triangle() > 1),
        ],
    ));

    verifiers[2] = Some(Verifier::from_description_and_options(
        "the □ number compared to 3",
        &[
            VerifierOption::from_description_and_closure("□ < 3", |code| code.square() < 3),
            VerifierOption::from_description_and_closure("□ = 3", |code| code.square() == 3),
            VerifierOption::from_description_and_closure("□ > 3", |code| code.square() > 3),
        ],
    ));

    verifiers[3] = Some(Verifier::from_description_and_options(
        "the □ number compared to 4",
        &[
            VerifierOption::from_description_and_closure("□ < 4", |code| code.square() < 4),
            VerifierOption::from_description_and_closure("□ = 4", |code| code.square() == 4),
            VerifierOption::from_description_and_closure("□ > 4", |code| code.square() > 4),
        ],
    ));

    verifiers[6] = Some(Verifier::from_description_and_options(
        "if ○ is even or odd",
        &[
            VerifierOption::from_description_and_closure("○ is even", |code| {
                code.circle() % 2 == 0
            }),
            VerifierOption::from_description_and_closure("○ is odd", |code| {
                code.circle() % 2 == 1
            }),
        ],
    ));

    verifiers[8] = Some(Verifier::from_description_and_options(
        "the number of 3s in the code",
        &[
            VerifierOption::from_description_and_closure("zero 3s", |code| {
                code.count_digit(3) == 0
            }),
            VerifierOption::from_description_and_closure("one 3", |code| code.count_digit(3) == 1),
            VerifierOption::from_description_and_closure("two 3s", |code| code.count_digit(3) == 2),
            VerifierOption::from_description_and_closure("three 3s", |code| {
                code.count_digit(3) == 3
            }),
        ],
    ));

    verifiers[9] = Some(Verifier::from_description_and_options(
        "the number of 4s in the code",
        &[
            VerifierOption::from_description_and_closure("zero 4s", |code| {
                code.count_digit(4) == 0
            }),
            VerifierOption::from_description_and_closure("one 4", |code| code.count_digit(4) == 1),
            VerifierOption::from_description_and_closure("two 4s", |code| code.count_digit(4) == 2),
            VerifierOption::from_description_and_closure("three 4s", |code| {
                code.count_digit(4) == 3
            }),
        ],
    ));

    verifiers[10] = Some(Verifier::from_description_and_options(
        "the △ number compared to the □ number",
        &[
            VerifierOption::from_description_and_closure("△ < □", |code| {
                code.triangle() < code.square()
            }),
            VerifierOption::from_description_and_closure("△ = □", |code| {
                code.triangle() == code.square()
            }),
            VerifierOption::from_description_and_closure("△ > □", |code| {
                code.triangle() > code.square()
            }),
        ],
    ));

    verifiers[12] = Some(Verifier::from_description_and_options(
        "the □ number compared to the ○ number",
        &[
            VerifierOption::from_description_and_closure("□ < ○", |code| {
                code.square() < code.circle()
            }),
            VerifierOption::from_description_and_closure("□ = ○", |code| {
                code.square() == code.circle()
            }),
            VerifierOption::from_description_and_closure("□ > ○", |code| {
                code.square() > code.circle()
            }),
        ],
    ));

    verifiers[13] = Some(Verifier::from_description_and_options(
        "which colour's number is smaller than either of the others",
        &[
            VerifierOption::from_description_and_closure("△ < □, ○", |code| {
                code.triangle() < code.square() && code.triangle() < code.circle()
            }),
            VerifierOption::from_description_and_closure("□ < △, ○", |code| {
                code.square() < code.triangle() && code.square() < code.circle()
            }),
            VerifierOption::from_description_and_closure("○ < □, △", |code| {
                code.circle() < code.square() && code.circle() < code.triangle()
            }),
        ],
    ));

    verifiers[15] = Some(Verifier::from_description_and_options(
        "the number of even numbers compared to the number of odd numbers",
        &[
            VerifierOption::from_description_and_closure("EVEN > ODD", |code| {
                code.count_even() >= 2
            }),
            VerifierOption::from_description_and_closure("EVEN < ODD", |code| {
                code.count_even() <= 1
            }),
        ],
    ));

    verifiers[22] = Some(Verifier::from_description_and_options(
        "the sum of all numbers compared to 6",
        &[
            VerifierOption::from_description_and_closure("△ + □ + ○ < 6", |code| {
                code.triangle() + code.square() + code.circle() < 6
            }),
            VerifierOption::from_description_and_closure("△ + □ + ○ = 6", |code| {
                code.triangle() + code.square() + code.circle() == 6
            }),
            VerifierOption::from_description_and_closure("△ + □ + ○ > 6", |code| {
                code.triangle() + code.square() + code.circle() > 6
            }),
        ],
    ));

    verifiers[32] = Some(Verifier::from_description_and_options(
        "that a specific colour is even or odd",
        &[
            VerifierOption::from_description_and_closure("△ is even", |code| {
                code.triangle() % 2 == 0
            }),
            VerifierOption::from_description_and_closure("△ is odd", |code| {
                code.triangle() % 2 == 1
            }),
            VerifierOption::from_description_and_closure("□ is even", |code| {
                code.square() % 2 == 0
            }),
            VerifierOption::from_description_and_closure("□ is odd", |code| {
                code.square() % 2 == 1
            }),
            VerifierOption::from_description_and_closure("○ is even", |code| {
                code.circle() % 2 == 0
            }),
            VerifierOption::from_description_and_closure("○ is odd", |code| {
                code.circle() % 2 == 1
            }),
        ],
    ));

    verifiers[33] = Some(Verifier::from_description_and_options(
        "which colour has the smallest number (or is tied for the smallest number)",
        &[
            VerifierOption::from_description_and_closure("△ <= □, ○", |code| {
                code.triangle() <= code.square() && code.triangle() <= code.circle()
            }),
            VerifierOption::from_description_and_closure("□ <= △, ○", |code| {
                code.square() <= code.triangle() && code.square() <= code.circle()
            }),
            VerifierOption::from_description_and_closure("○ <= □, △", |code| {
                code.circle() <= code.square() && code.circle() <= code.triangle()
            }),
        ],
    ));

    verifiers[44] = Some(Verifier::from_description_and_options(
        "how many 1s OR how many 3s there are in the code",
        &[
            VerifierOption::from_description_and_closure("zero 1s", |code| {
                code.count_digit(1) == 0
            }),
            VerifierOption::from_description_and_closure("one 1s", |code| code.count_digit(1) == 1),
            VerifierOption::from_description_and_closure("two 1s", |code| code.count_digit(1) == 2),
            VerifierOption::from_description_and_closure("zero 3s", |code| {
                code.count_digit(3) == 0
            }),
            VerifierOption::from_description_and_closure("one 3s", |code| code.count_digit(3) == 1),
            VerifierOption::from_description_and_closure("two 3s", |code| code.count_digit(3) == 2),
        ],
    ));

    verifiers[number - 1]
        .clone()
        .expect("Verifier not implemented")
}

const MAX_VERIFIER_OPTIONS: usize = 9;

#[derive(Copy, Clone)]
pub struct VerifierOption {
    pub(crate) description: &'static str,
    code_set: CodeSet,
}

impl VerifierOption {
    pub fn code_set(&self) -> CodeSet {
        self.code_set
    }
}

pub(crate) trait Intersection {
    type To;
    fn intersect(self) -> Self::To;
}

impl<T: Iterator<Item = CodeSet>> Intersection for T {
    type To = CodeSet;
    fn intersect(self) -> Self::To {
        self.fold(CodeSet::all(), |still_possible_codes, new_code_set| {
            still_possible_codes.intersected_with(new_code_set)
        })
    }
}

#[derive(Clone)]
pub struct Verifier {
    description: &'static str,
    options: ArrayVec<VerifierOption, MAX_VERIFIER_OPTIONS>,
}

impl Debug for Verifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.description)?;
        for option in self.options.iter() {
            writeln!(f, "- {}", option.description)?;
        }
        Ok(())
    }
}

impl Verifier {
    pub fn description(&self) -> &'static str {
        self.description
    }

    pub fn number_of_options(&self) -> usize {
        self.options.len()
    }

    pub fn from_description_and_options(
        description: &'static str,
        options: &[VerifierOption],
    ) -> Self {
        Verifier {
            description,
            options: options.iter().copied().collect(),
        }
    }

    pub fn option(&self, choice: u8) -> &VerifierOption {
        &self.options[choice as usize]
    }

    pub fn options(&self) -> impl Iterator<Item = &VerifierOption> + '_ {
        self.options.iter()
    }
}

impl VerifierOption {
    pub fn from_description_and_closure(
        description: &'static str,
        checker: fn(Code) -> bool,
    ) -> VerifierOption {
        VerifierOption {
            description,
            code_set: CodeSet::from_closure(checker),
        }
    }
}
