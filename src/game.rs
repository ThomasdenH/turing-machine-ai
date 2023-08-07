use std::iter;

use arrayvec::ArrayVec;

use crate::{
    code::CodeSet,
    verifier::{Intersection, Verifier, VerifierOption},
};

const MAX_VERIFIERS: usize = 6;

/// A game layout, consisting of the chosen verifiers.
pub struct Game {
    verifiers: Vec<Verifier>,
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
    pub fn verifier_count(self) -> usize {
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

    pub fn new_from_verifiers(verifiers: Vec<Verifier>) -> Game {
        Game { verifiers }
    }

    /// Get all assignments, regardness of their validity.
    pub fn all_assignments(&self) -> impl Iterator<Item = Assignment> + '_ {
        let len = self.verifiers.len();
        iter::successors(
            Some(Assignment {
                choice: ArrayVec::from_iter(iter::repeat(0).take(len)),
            }),
            move |prev| {
                let mut new = prev.clone();
                new.choice[0] += 1;
                for index in 0..len {
                    // Carry to the right
                    if new.choice[index] >= self.verifiers[index].number_of_options() as u8 {
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
    pub fn is_possible_solution(&self, assignment: &Assignment) -> bool {
        if self.possible_codes_for_assignment(assignment).size() != 1 {
            return false;
        }

        // Test for reduncancy
        for excluded_verifier in 0..self.verifiers.len() {
            let possible_codes = self.verifier_options_for_assignment(assignment)
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
}
