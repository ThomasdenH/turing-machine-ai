use std::collections::HashSet;

use arrayvec::ArrayVec;
use itertools::Itertools;

use crate::game::{Assignment, Game};

use self::state::ChoosingVerfier;

mod state {
    use crate::game::Assignment;

    pub(crate) struct Human;
    pub(crate) struct Verifier;
}

trait GameStateLike {
    type PossibleMove;
    type NextGameState;
    fn possible_moves(&self) -> impl Iterator<Item = PossibleMove> + '_;
    fn do_move(&self, move_to_do: Self::PossibleMove) -> Self::NextGameState;
}

struct GameState<'a, T> {
    /// The current set of verifiers.
    game: &'a Game,
    /// All possible verifier assignments that consitute a solution to the
    /// problem.
    all_possible_solutions: Vec<Assignment>,
    is_still_possible: BitMap<125>,
    current_chosen_assignment: Option<Assignment>,
    /// The current round.
    round: u16,
    /// How many verifiers have been checked in total.
    total_verifiers_checked: u16
}

enum HumanMove {
    CodeAndVerifier(Assignment, usize),
    Verifier(usize)
}

impl<'a> GameStateLike for GameState<'a, state::Human> {
    type PossibleMove = HumanMove;
    type NextGameState = GameState<'a, state::Verifier>;

    fn possible_moves(&self) -> impl Iterator<Item = PossibleMove> + '_ {
        if self.current_chosen_assignment.is_none() {
            // The possible moves consist of all possible assignments that can be checked.
            let mut possible_assignments = vec![HashSet::new(), self.assignments.len()];
            for assignment in self.assignments {
                for (choice, possible_assignments) in assignment.choices().zip(possible_assignments.iter_mut()) {
                    possible_assignments.insert(choice);
                }
            }
            possible_assignments.iter()
                .map(|i| i.iter())
                .multi_cartesian_product()
                .map(|choice| Assignment::from_choices(choice))
                // Finally choose a verifier
                .flat_map(|choice| (0..self.game.verifier_count()).map(|i| HumanMove::CodeAndVerifier(choice, i)))
        }
    }

    fn do_move(&self, move_to_do: Assignment) -> GameState<state::ChoosingVerfier<0>> {
        GameState {
            game: self.game,
            assignments: self.assignments,
            state: state::ChoosingVerfier {
                current_assignment: move_to_do,
            },
        }
    }
}

impl<'a> GameStateLike for GameState<'a, state::ChoosingVerfier<0>> {
    type PossibleMove = usize;
    type NextGameState = GameState<'a, state::VerifierSolution>;
    fn do_move(&self, move_to_do: Self::PossibleMove) -> Self::NextGameState {
        GameState {
            game: self.game,
            assignments: self.assignments,
            state: state::VerifierSolution
        }
    }
}
