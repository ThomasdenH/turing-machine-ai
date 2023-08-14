use std::fmt::Debug;

use crate::{
    code::{BitCode, CodeSet},
    game::Game,
};

/// A struct representing the current game state. It contains the possible
/// solutions for the verifier selection, the currently selected code (if any),
/// the currently selected verifier (if any), whether a verifier was tested, as
/// well as how many tests were performed.
#[derive(Copy, Clone)]
pub struct State<'a> {
    game: &'a Game,
    /// All the codes that are still possible solutions.
    possible_codes: CodeSet,
    currently_selected_code: Option<BitCode>,
    currently_chosen_verifier_option: Option<ChosenVerifierOption>,
    guessed_one_verifier_for_code: bool,
    codes_guessed: u8,
    verifiers_checked: u8,
}

/// A struct representing a score associated with a game state. The score
/// represents how desirable it is for the guesser. A perfect score compares
/// the highest whereas a game without a solution is the lowest possible score.
///
/// Internally, the score is inverted so that a perfect game is represented by a 0.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct StateScore(u16);

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct GameScore {
    pub codes_guessed: u8,
    pub verifiers_checked: u8,
}

impl StateScore {
    fn no_solution() -> Self {
        // A state without a solution gives the best possible score, which is
        // the worst possible score for the answer to the verifier. This
        // ensures that they will never pick this result. Instead they will
        // give `StateScore::useless_verifier_check()`, which is the worst
        // possible outcome for the player.
        StateScore(0)
    }

    fn solution(codes_guessed: u8, verifier_checks: u8) -> Self {
        StateScore(u16::from(codes_guessed) << 8 | u16::from(verifier_checks))
    }

    /// This is represented by the worst possible outcome for the verifier.
    /// This is actually a heuristic---because it is never a good idea to guess
    /// without gaining information, these branches do not have to be explored.
    fn useless_verifier_check() -> Self {
        StateScore(u16::MAX)
    }

    fn perfect_game() -> Self {
        StateScore(0)
    }

    pub fn codes_and_verifiers_checked(self) -> Option<GameScore> {
        if self.0 == u16::MAX {
            None
        } else {
            Some(GameScore {
                codes_guessed: (self.0 >> 8) as u8,
                verifiers_checked: (self.0 & 0b1111_1111) as u8,
            })
        }
    }

    fn min() -> Self {
        StateScore(u16::MAX)
    }

    fn max() -> Self {
        StateScore(u16::MIN)
    }
}

pub enum DoMoveResult<'a> {
    NoCodesLeft,
    UselessVerifierCheck,
    State(State<'a>),
}

impl Ord for StateScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0).reverse()
    }
}

impl PartialOrd for StateScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A verifier answer, represented either by a cross or a check.
#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum VerifierSolution {
    Cross,
    Check,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct ChosenVerifierOption(u8);

impl From<u8> for ChosenVerifierOption {
    fn from(value: u8) -> Self {
        ChosenVerifierOption(value)
    }
}

impl Debug for ChosenVerifierOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", ('A'..).nth(self.0.into()).unwrap())
    }
}

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Move {
    ChooseNewCode(BitCode),
    VerifierSolution(VerifierSolution),
    ChooseVerifierOption(ChosenVerifierOption),
}

impl Move {
    fn is_maximizing_score(&self) -> bool {
        !matches!(self, Move::VerifierSolution(_))
    }
}

impl<'a> State<'a> {
    pub fn new(game: &'a Game) -> Self {
        State {
            game,
            possible_codes: game.possible_solutions(),
            currently_selected_code: None,
            currently_chosen_verifier_option: None,
            guessed_one_verifier_for_code: false,
            codes_guessed: 0,
            verifiers_checked: 0,
        }
    }

    pub fn possible_codes(&self) -> CodeSet {
        self.possible_codes
    }

    pub fn is_solved(self) -> bool {
        self.possible_codes.size() == 1
    }

    /// Return the state after performing the given move.
    pub fn after_move(mut self, move_to_do: Move) -> DoMoveResult<'a> {
        match move_to_do {
            Move::ChooseNewCode(code) => {
                self.currently_selected_code = Some(code);
                self.codes_guessed += 1;
                self.guessed_one_verifier_for_code = false;
            }
            Move::ChooseVerifierOption(choose_verifier_option) => {
                debug_assert!(self.currently_chosen_verifier_option.is_none());
                self.currently_chosen_verifier_option = Some(choose_verifier_option);
                self.verifiers_checked += 1;
            }
            Move::VerifierSolution(verifier_solution) => {
                // Eliminate codes
                let chosen_verifier = self.currently_chosen_verifier_option.unwrap().0;

                // Get all codes that correspond to a verifier option giving the provided answer.
                let bitmask_for_solution = self
                    .game
                    .verfier(chosen_verifier)
                    .options()
                    .map(|verifier_option| verifier_option.code_set())
                    .filter(|code_set| {
                        let would_give_check =
                            code_set.contains_bit_code(self.currently_selected_code.unwrap());
                        let gives_check = verifier_solution == VerifierSolution::Check;
                        would_give_check == gives_check
                    })
                    .collect::<CodeSet>();
                let possible_codes = self.possible_codes;
                let new_possible_codes = possible_codes.intersected_with(bitmask_for_solution);
                if new_possible_codes == possible_codes {
                    return DoMoveResult::UselessVerifierCheck;
                } else if new_possible_codes.size() == 0 {
                    return DoMoveResult::NoCodesLeft;
                } else {
                    self.possible_codes = new_possible_codes;
                }

                self.currently_chosen_verifier_option = None;
                if self.verifiers_checked == 3 {
                    self.currently_selected_code = None;
                }
            }
        }
        DoMoveResult::State(self)
    }

    /// Returns true if the game is awaiting a verifier answer.
    pub fn is_awaiting_result(&self) -> bool {
        self.currently_chosen_verifier_option.is_some()
    }

    /// Return all possible moves. Notably these are not verified in every way:
    /// - Verifiers may return impossible results, leading to no solution.
    /// - Codes or verifiers may be chosen that do not provide information to
    ///   the player.
    pub fn possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        if self.is_awaiting_result() {
            // Already chosen a verifier option, resolve
            moves.append(&mut vec![
                Move::VerifierSolution(VerifierSolution::Check),
                Move::VerifierSolution(VerifierSolution::Cross),
            ]);
        } else {
            if self.currently_selected_code.is_some() {
                // Selected a code, so now select a verifier.
                moves.append(
                    &mut (0..self.game.verifier_count())
                        .map(|i| Move::ChooseVerifierOption(ChosenVerifierOption(i as u8)))
                        .collect(),
                );
            }
            if self.currently_selected_code.is_none() || self.guessed_one_verifier_for_code {
                // If no code was selected, or if a verifier was checked, the
                // next code may be selected.
                moves.append(
                    &mut CodeSet::all()
                        .iter_bit_code()
                        .map(Move::ChooseNewCode)
                        .collect::<Vec<Move>>(),
                );
            }
        }
        moves
    }

    /// Perform minmax with alpha-beta pruning.
    fn alphabeta(
        self,
        mut alpha: StateScore,
        mut beta: StateScore,
        is_maximizing_score: bool,
    ) -> (StateScore, Option<Move>) {
        // println!("{:?}, {:?}", alpha, beta);
        // println!("Current code: {:?}", self.currently_selected_code);
        // println!("Codes: {}, verifiers: {}", self.codes_guessed, self.verifiers_checked);
        // println!("{:?}", self.possible_codes);
        // If the game is solved, return the result.
        if self.is_solved() {
            (
                StateScore::solution(self.codes_guessed, self.verifiers_checked),
                None,
            )
        } else if is_maximizing_score {
            let mut highest_score = StateScore::min();
            let mut best_move = None;
            for move_to_do in self.possible_moves() {
                let next_node = self.after_move(move_to_do);
                let score = match next_node {
                    DoMoveResult::NoCodesLeft => StateScore::no_solution(),
                    DoMoveResult::UselessVerifierCheck => StateScore::useless_verifier_check(),
                    DoMoveResult::State(state) => {
                        state
                            .alphabeta(alpha, beta, move_to_do.is_maximizing_score())
                            .0
                    }
                };
                if score > highest_score {
                    highest_score = score;
                    best_move = Some(move_to_do);
                }
                if score > beta {
                    break;
                }
                if score > alpha {
                    alpha = score;
                }
            }
            (highest_score, best_move)
        } else {
            let mut lowest_score = StateScore::max();
            for move_to_do in self.possible_moves() {
                let next_node = self.after_move(move_to_do);
                let score = match next_node {
                    DoMoveResult::NoCodesLeft => StateScore::no_solution(),
                    DoMoveResult::UselessVerifierCheck => StateScore::useless_verifier_check(),
                    DoMoveResult::State(state) => {
                        state
                            .alphabeta(alpha, beta, move_to_do.is_maximizing_score())
                            .0
                    }
                };
                if score < lowest_score {
                    lowest_score = score;
                }
                if score < alpha {
                    break;
                }
                if score < beta {
                    beta = score;
                }
            }
            // It doesn't make sense to return a move for the other player
            (lowest_score, None)
        }
    }

    /// Find the best possible move to minimize the maximum amount of codes and
    /// verifier checks needed. The game must be at a state where the player
    /// chooses a code or a verifier.
    pub fn find_best_move(self) -> (GameScore, Move) {
        assert!(!self.is_awaiting_result() && !self.is_solved());
        // The optimal possible game.
        let alpha = StateScore::perfect_game();
        // The worst possible game.
        let beta = StateScore::useless_verifier_check();
        if let (score, Some(move_to_do)) = self.alphabeta(alpha, beta, true) {
            (score.codes_and_verifiers_checked().unwrap(), move_to_do)
        } else {
            panic!("No move possible");
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::gametree::GameScore;

    use super::StateScore;

    #[test]
    fn test_game_score() {
        for codes_guessed in 0..10 {
            for verifiers_checked in codes_guessed..30 {
                let state_score = StateScore::solution(codes_guessed, verifiers_checked);
                assert_eq!(
                    state_score.codes_and_verifiers_checked().unwrap(),
                    GameScore {
                        codes_guessed,
                        verifiers_checked
                    }
                );
            }
        }
    }
}
