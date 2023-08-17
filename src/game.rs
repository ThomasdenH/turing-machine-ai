use std::{fmt::Debug, iter};

use arrayvec::ArrayVec;

use crate::{
    code::CodeSet,
    verifier::{Intersection, Verifier, VerifierOption, get_verifier_by_number}, gametree::State,
};

/// The maximum amount of verifiers allowed in a game.
const MAX_VERIFIERS: usize = 6;

/// A game layout, consisting of the chosen verifiers.
pub struct Game {
    verifiers: Vec<Verifier>,
}

impl Debug for Game {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (verifier, letter) in self.verifiers.iter().zip('A'..) {
            writeln!(f, "Verifier {letter}")?;
            writeln!(f, "{verifier:?}")?;
        }
        Ok(())
    }
}

/// A particular assignment for a game. For example, this might indicate that
/// for the first verifier, the second option is selected, for the second
/// verifier the third option, etc.
#[derive(Clone, Debug)]
pub struct Assignment {
    choice: ArrayVec<u8, MAX_VERIFIERS>,
}

impl Assignment {
    /// Go through all chosen options, in the order of the verifiers.
    pub fn choices(&self) -> impl Iterator<Item = u8> + '_ {
        self.choice.iter().copied()
    }

    /// Create an assignment from the individual choices.
    pub fn from_choices<T: Into<ArrayVec<u8, MAX_VERIFIERS>>>(choices: T) -> Self {
        Assignment {
            choice: choices.into(),
        }
    }
}

impl Game {
    pub fn starting_state(&self) -> State<'_> {
        State::new(self)
    }

    // TODO: Index by custom type?
    pub fn verfier(&self, index: u8) -> &Verifier {
        &self.verifiers[index as usize]
    }

    pub fn verifier_count(&self) -> usize {
        self.verifiers.len()
    }

    pub fn verifier_options_for_assignment<'a, 'b: 'a>(
        &'a self,
        assignment: &'b Assignment,
    ) -> impl Iterator<Item = VerifierOption> + 'a {
        self.verifiers
            .iter()
            .zip(assignment.choices())
            .map(|(verifier, choice)| *verifier.option(choice))
    }

    #[must_use]
    pub fn new_from_verifiers(verifiers: Vec<Verifier>) -> Game {
        Game { verifiers }
    }

    pub fn new_from_verifier_numbers(verifier_numbers: impl Iterator<Item = usize>) -> Game {
        Game { verifiers: verifier_numbers.map(get_verifier_by_number).collect() }
    }

    /// Get all assignments, regardless of their validity.
    pub fn all_assignments(&self) -> impl Iterator<Item = Assignment> + '_ {
        let len = self.verifiers.len();
        iter::successors(
            Some(Assignment {
                choice: iter::repeat(0).take(len).collect(),
            }),
            move |prev| {
                let mut new = prev.clone();
                new.choice[0] += 1;
                for index in 0..len {
                    // Carry to the right
                    if usize::from(new.choice[index]) >= self.verifiers[index].number_of_options() {
                        new.choice[index] = 0;
                        if index + 1 >= len {
                            return None;
                        }
                        new.choice[index + 1] += 1;
                    }
                }
                Some(new)
            },
        )
    }

    /// Get all codes that adhere to a particular assignment.
    pub fn possible_codes_for_assignment(&self, assignment: &Assignment) -> CodeSet {
        self.verifiers
            .iter()
            .zip(assignment.choices())
            .map(|(verifier, choice)| verifier.option(choice).code_set())
            .intersect()
    }

    pub fn print_assigment(&self, assignment: &Assignment) {
        for (verifier, assignment) in self.verifiers.iter().zip(assignment.choice.iter()) {
            println!("{}", verifier.description());
            println!("- {}", verifier.option(*assignment).description);
        }
    }

    /// Check if the assignment is a possible puzzle solution. This means that
    /// there should be a single code that adheres to the verifiers, and that
    /// none of the verifiers are redundant.
    #[must_use]
    pub fn is_possible_solution(&self, assignment: &Assignment) -> bool {
        if self.possible_codes_for_assignment(assignment).size() != 1 {
            return false;
        }

        // Test for reduncancy
        for excluded_verifier in 0..self.verifiers.len() {
            let possible_codes = self
                .verifier_options_for_assignment(assignment)
                .enumerate()
                .filter_map(|(index, verifier_and_choice)| {
                    Some(verifier_and_choice).filter(|_| index != excluded_verifier)
                })
                .map(|verifier_option| verifier_option.code_set())
                .intersect();
            if possible_codes.size() <= 1 {
                return false;
            }
        }
        true
    }

    /// Get all possible solutions, i.e. those codes that correspond to a
    /// verifier result that have exactly one solution.
    pub fn possible_solutions(&self) -> CodeSet {
        self.all_assignments()
            .filter(|assignment| self.is_possible_solution(assignment))
            .map(|assignment| self.possible_codes_for_assignment(&assignment))
            .fold(CodeSet::empty(), |all, new| all.union_with(new))
    }
}
