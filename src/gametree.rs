use std::collections::HashSet;

use arrayvec::ArrayVec;
use itertools::Itertools;

use crate::game::{Assignment, Game};

use self::state::ChoosingVerfier;

mod state {
    use crate::game::Assignment;

    pub(crate) struct ChoosingCode;
    pub(crate) struct ChoosingVerfier<const VERIFIER_TO_CHOOSE: usize> {
        current_assignment: Assignment,
    }
    pub(crate) struct VerifierSolution;
}

trait GameStateLike {
    type PossibleMove;
    type NextGameState;
    fn possible_moves(&self) -> impl Iterator<Item = PossibleMove> + '_;
    fn do_move(&self, move_to_do: Self::PossibleMove) -> Self::NextGameState;
}

struct GameState<'a, T> {
    game: &'a Game,
    assignments: &'a ArrayVec<Assignment, MAX_VERIFIERS>,
    state: T,
}

impl<'a> GameStateLike for GameState<'a, state::ChoosingCode> {
    type PossibleMove = Assignment;
    type NextGameState = GameState<'a, state::ChoosingVerfier<0>>;

    fn possible_moves(&self) -> impl Iterator<Item = PossibleMove> + '_ {
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
