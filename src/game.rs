//! This module contains code related to a full game but not to game state.
//!
//! In other words, deductions based on verifiers are performed here, but no
//! logic for checking codes and verifiers.

use std::fmt::Debug;

use crate::{
    code::{Code, Set},
    gametree::VerifierSolution,
    verifier::{get_verifier_by_number, Intersection, Verifier, VerifierOption},
};

/// A game layout, consisting of the chosen verifiers.
#[derive(Clone, Eq, PartialEq, Hash)]
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

const ASSIGNMENT_BITS_PER_VERIFIER: usize = 16;

#[derive(Eq, PartialEq, Clone, Copy, Hash)]
pub struct Assignment {
    // Representation: 6*18 bits, for each option.
    // Offset is 16 * verifier index.
    bitmap: u128,
}

impl Debug for Assignment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, choice) in self.choices().enumerate() {
            let verifier = ChosenVerifier::from(index);
            write!(f, "{verifier:?}: {choice} ")?;
        }
        Ok(())
    }
}

impl Assignment {
    /// Go through all chosen options, in the order of the verifiers.
    pub fn choices(&self) -> impl Iterator<Item = u8> + '_ {
        (0..6 * ASSIGNMENT_BITS_PER_VERIFIER)
            .step_by(ASSIGNMENT_BITS_PER_VERIFIER)
            .map(|verifier_start|
                // Find assignment for this verifier
                (0..ASSIGNMENT_BITS_PER_VERIFIER)
                    .find(|bit_for_option| self.bitmap & (1 << (verifier_start + bit_for_option)) != 0)
            )
            .map_while(|maybe_verifier| maybe_verifier)
            .map(|i| i.try_into().unwrap())
    }

    pub fn mask_for_verifier_and_response(verifier: usize, response: usize) -> u128 {
        1 << (verifier * ASSIGNMENT_BITS_PER_VERIFIER + response)
    }

    /// Create an assignment from the individual choices.
    pub fn from_choices<T: Iterator<Item = usize>>(choices: T) -> Self {
        Assignment {
            bitmap: choices
                .enumerate()
                .map(|(index, choice)| Self::mask_for_verifier_and_response(index, choice))
                .fold(0u128, |acc, x| acc | x),
        }
    }
}

struct AllAssignmentsIterator<'a> {
    choice: Vec<usize>,
    game: &'a Game,
}

impl<'a> AllAssignmentsIterator<'a> {
    pub fn new(game: &'a Game) -> Self {
        let len = game.verifier_count();
        assert!((4..=6).contains(&len));
        Self {
            choice: vec![0; len],
            game,
        }
    }
}

impl<'a> Iterator for AllAssignmentsIterator<'a> {
    type Item = Assignment;
    fn next(&mut self) -> Option<Self::Item> {
        self.choice[0] += 1;
        for (index, verifier) in self.game.verifiers.iter().enumerate() {
            // Carry to the right
            if self.choice[index] >= verifier.number_of_options() {
                self.choice[index] = 0;
                if index + 1 >= self.choice.len() {
                    return None;
                }
                self.choice[index + 1] += 1;
            }
        }
        Some(Assignment::from_choices(self.choice.iter().copied()))
    }
}

/// Represents a choice of verifier, i.e. verifier 'B'.
///
/// # Example
/// ```
/// use turing_machine_ai::game::ChosenVerifier;
///
/// let verifier = ChosenVerifier::from(1usize);
/// assert_eq!(format!("{verifier:?}"), "B");
/// ```
#[derive(Eq, PartialEq, Copy, Clone, Hash)]
pub struct ChosenVerifier(usize);

impl From<usize> for ChosenVerifier {
    fn from(value: usize) -> Self {
        ChosenVerifier(value)
    }
}

impl Debug for ChosenVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ('A'..).nth(self.0).unwrap())
    }
}

impl Game {
    #[must_use]
    pub fn verfier(&self, index: ChosenVerifier) -> &Verifier {
        &self.verifiers[index.0]
    }

    pub fn iter_verifier_choices(&self) -> impl Iterator<Item = ChosenVerifier> {
        (0..self.verifiers.len()).map(ChosenVerifier)
    }

    /// Return the number of verifiers for this game.
    ///
    /// # Example
    /// ```
    /// use turing_machine_ai::game::Game;
    ///
    /// let game = Game::new_from_verifier_numbers([2, 14, 17, 21, 22].iter().copied());
    /// assert_eq!(game.verifier_count(), 5);
    /// ```
    #[must_use]
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

    #[must_use]
    pub fn new_from_verifier_numbers(verifier_numbers: impl Iterator<Item = usize>) -> Game {
        Game {
            verifiers: verifier_numbers.map(get_verifier_by_number).collect(),
        }
    }

    /// Get all codes that adhere to a particular assignment.
    #[must_use]
    pub fn possible_codes_for_assignment(&self, assignment: &Assignment) -> Set {
        self.verifiers
            .iter()
            .zip(assignment.choices())
            .map(|(verifier, choice)| verifier.option(choice).code_set())
            .intersect()
    }

    pub fn print_assigment(&self, assignment: &Assignment) {
        for (verifier, assignment) in self.verifiers.iter().zip(assignment.choices()) {
            println!("{}", verifier.description());
            println!("- {}", verifier.option(assignment).description);
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
    #[must_use]
    pub fn possible_solutions(&self) -> PossibleSolutions {
        let assignments_and_codes = AllAssignmentsIterator::new(self)
            .filter(|assignment| self.is_possible_solution(assignment))
            .map(|assignment| {
                let code = self
                    .possible_codes_for_assignment(&assignment)
                    .into_iter()
                    .next()
                    .unwrap();
                (assignment, code)
            })
            .collect();
        PossibleSolutions {
            assignments_and_codes,
        }
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
pub struct PossibleSolutions {
    assignments_and_codes: Vec<(Assignment, Code)>,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct PossibleSolutionFilter<'a> {
    possible_solutions: &'a PossibleSolutions,
    containing: u128,
}

impl<'a> Debug for PossibleSolutionFilter<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (_bit, assignment, code) in self.possible_codes_with_index() {
            writeln!(f, "{assignment:?} -> {code:?}")?;
        }
        Ok(())
    }
}

impl<'a> From<&'a PossibleSolutions> for PossibleSolutionFilter<'a> {
    fn from(possible_solutions: &'a PossibleSolutions) -> Self {
        let containing = std::iter::successors(Some(1u128), |prev| Some(prev << 1))
            .take(possible_solutions.assignments_and_codes.len())
            .fold(0, |acc, x| acc | x);
        PossibleSolutionFilter {
            possible_solutions,
            containing,
        }
    }
}

impl<'a> PossibleSolutionFilter<'a> {
    pub fn size(&self) -> u32 {
        self.containing.count_ones()
    }

    pub fn is_empty(self) -> bool {
        self.containing == 0
    }

    pub fn possible_codes(&self) -> impl Iterator<Item = Code> + '_ {
        self.possible_codes_with_index()
            .map(|(_bit, _assignment, code)| code)
    }

    fn possible_codes_with_index(&self) -> impl Iterator<Item = (u128, Assignment, Code)> + '_ {
        std::iter::successors(Some(1u128), |prev| Some(prev << 1))
            .zip(self.possible_solutions.assignments_and_codes.iter())
            .filter_map(|(bit, (assignment, code))| {
                Some((bit, *assignment, *code)).filter(|_| self.containing & bit != 0)
            })
    }

    /// If just one code is left, return it. Notably, this code may not have a
    /// unique assignment.
    pub fn solution(&self) -> Option<Code> {
        let mut only_code = None;
        for code in self.possible_codes() {
            if let Some(only_code) = only_code {
                if only_code != code {
                    return None;
                }
            }
            only_code = Some(code)
        }
        only_code
    }

    pub fn filter_through_verifier_check(
        mut self,
        game: &Game,
        verifier: ChosenVerifier,
        code: Code,
        solution: VerifierSolution,
    ) -> Self {
        // If the verifier gives a check, the solution must give a check at one
        // place where the current code gives a check. In other words, we can
        // intersect the possible codes with the union of all verifier options
        // that this code satisfies.
        //
        // Example: The verifier checks that a specific colour is less than 4.
        //      The guess is (△, □, ○) = (3, 3, 5).
        //      If the verifier gives a check, the possible solutions are those
        //      those that have either △ < 4 or □ < 4 (or both).
        //
        // If the verifier gives a cross, then the solution must satisfy one of
        // the other criteria that this code doesn't. This does not mean that
        // the criteria that was currenly tested is incorrect, but we know that
        // at least one of the other criteria is correct. We can therefore
        // intersect the possible codes with the union of all verifier options
        // that this code does not satisfy.
        //
        // Example: If in the example above the verifier would give a cross,
        //      we do not know that △ >= 4 or □ >= 4; we instead know that
        //      ○ < 4.

        // Get all options for this verifier that give the answer. If we have a
        // check, select all options that have a check. If we have no check,
        // select all options that have no check. The solution must have one of
        // the selected options.
        let matching_options = self.mask_for_verifier_check(game, verifier, code, solution);

        // Now remove all options that are no longer possible.
        for (index, bit) in std::iter::successors(Some(1u128), |prev| Some(prev << 1))
            .take(self.possible_solutions.assignments_and_codes.len())
            .enumerate()
        {
            let (assignment, _code) = self.possible_solutions.assignments_and_codes[index];
            // Check if it is in!
            if self.containing & bit != 0 {
                // Does not match the new info
                if assignment.bitmap & matching_options == 0 {
                    // Remove
                    self.containing &= !bit;
                }
            }
        }
        self
    }

    fn mask_for_verifier_check(
        self,
        game: &Game,
        verifier: ChosenVerifier,
        code: Code,
        solution: VerifierSolution,
    ) -> u128 {
        let gives_check = solution == VerifierSolution::Check;
        let verifier_start = verifier.0 * ASSIGNMENT_BITS_PER_VERIFIER;
        game.verfier(verifier)
            .options()
            .enumerate()
            .map(|(index, option)| (index, option.code_set().contains(code)))
            .filter(|(_index, would_give_solution)| *would_give_solution == gives_check)
            .map(|(index, _)| 1 << (verifier_start + index))
            .fold(0, |acc, x| acc | x)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct PossibleAssignments {
    bitmap: u128
}

impl PossibleAssignments {

}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{
        code::{Code, Set},
        game::ChosenVerifier, gametree::VerifierSolution,
    };

    use super::{Game, PossibleSolutionFilter};

    #[test]
    fn test_filter() -> Result<(), Box<dyn Error>> {
        let game = Game::new_from_verifier_numbers([3, 7, 10, 14].iter().copied());
        let possible_solutions = game.possible_solutions();
        let possible_solutions_filter = PossibleSolutionFilter::from(&possible_solutions);
        assert_eq!(
            possible_solutions_filter.possible_codes().collect::<Set>(),
            [
                Code::from_digits(4, 3, 1)?,
                Code::from_digits(1, 3, 2)?,
                Code::from_digits(4, 3, 2)?,
                Code::from_digits(1, 5, 2)?,
                Code::from_digits(5, 3, 4)?,
                Code::from_digits(4, 3, 5)?,
            ]
            .iter()
            .copied()
            .collect::<Set>()
        );

        Ok(())
    }

    #[test]
    fn test_filter_2() -> Result<(), Box<dyn Error>> {
        let game = Game::new_from_verifier_numbers([18, 21, 37, 48].iter().copied());
        let possible_solutions = game.possible_solutions();
        let possible_solutions_filter = PossibleSolutionFilter::from(&possible_solutions);
        assert_eq!(
            possible_solutions_filter.possible_codes().collect::<Set>(),
            [
                Code::from_digits(1, 3, 5)?,
                Code::from_digits(1, 5, 3)?,
                Code::from_digits(3, 1, 5)?,
                Code::from_digits(3, 5, 1)?,
                Code::from_digits(5, 1, 3)?,
                Code::from_digits(5, 3, 1)?,
            ]
            .iter()
            .copied()
            .collect::<Set>()
        );

        let verifier = ChosenVerifier(3);
        let code = Code::from_digits(1, 2, 3)?;
        let solution = VerifierSolution::Check;
        println!("{:0128b}", possible_solutions_filter.mask_for_verifier_check(&game, verifier, code, solution));
        for assignment in possible_solutions_filter.possible_codes_with_index().map(|(_, assignment, _)| assignment) {
            println!("{:0128b}", assignment.bitmap);

        }

        let possible_solutions_filter = possible_solutions_filter.filter_through_verifier_check(
            &game,
            ChosenVerifier(3),
            Code::from_digits(1, 2, 3)?,
            VerifierSolution::Check,
        );

        println!("{:0128b}", possible_solutions_filter.mask_for_verifier_check(&game, verifier, code, solution));
        for assignment in possible_solutions_filter.possible_codes_with_index().map(|(_, assignment, _)| assignment) {
            println!("{:0128b}", assignment.bitmap);

        }

        assert_eq!(
            possible_solutions_filter.possible_codes().collect::<Set>(),
            [
                Code::from_digits(5, 1, 3)?,
                Code::from_digits(1, 5, 3)?,
                Code::from_digits(1, 3, 5)?
            ]
            .iter()
            .copied()
            .collect::<Set>()
        );

        Ok(())
    }
}
