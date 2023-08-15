use std::fmt::Debug;

use thiserror::Error;

use crate::{
    code::{BitCode, CodeSet},
    game::Game,
};

/// A struct representing the current game state. It contains the possible
/// solutions for the verifier selection, the currently selected code (if any),
/// the currently selected verifier (if any), whether a verifier was tested, as
/// well as how many tests were performed.
#[derive(Copy, Clone, Debug)]
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

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum AfterMoveInfo {
    UselessVerifierCheck
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

#[derive(Copy, Clone, Eq, PartialEq, Error, Debug)]
pub enum AfterMoveError {
    #[error("invalid move for game state")]
    InvalidMoveError,
    #[error("there are no solutions left for this game state")]
    NoCodesLeft
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

    /// Return the state after performing the given move. If the given move is
    /// invalid, this function returns an error. In addition to the state
    /// itself, this function will sometimes provide additional information
    /// about the transition as the second argument of the tuple.
    pub fn after_move(mut self, move_to_do: Move) -> Result<(State<'a>, Option<AfterMoveInfo>), AfterMoveError> {
        let mut info = None;
        match move_to_do {
            Move::ChooseNewCode(code) => {
                if self.is_awaiting_result() {
                    return Err(AfterMoveError::InvalidMoveError);
                }
                self.currently_selected_code = Some(code);
                self.codes_guessed += 1;
                self.guessed_one_verifier_for_code = false;
            }
            Move::ChooseVerifierOption(choose_verifier_option) => {
                if self.is_awaiting_result() {
                    return Err(AfterMoveError::InvalidMoveError);
                }
                self.currently_chosen_verifier_option = Some(choose_verifier_option);
                self.verifiers_checked += 1;
            }
            Move::VerifierSolution(verifier_solution) => {
                if !self.is_awaiting_result() {
                    return Err(AfterMoveError::InvalidMoveError);
                }
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
                    info = Some(AfterMoveInfo::UselessVerifierCheck);
                } else if new_possible_codes.size() == 0 {
                    return Err(AfterMoveError::NoCodesLeft);
                } else {
                    self.possible_codes = new_possible_codes;
                }

                self.currently_chosen_verifier_option = None;
                if self.verifiers_checked == 3 {
                    self.currently_selected_code = None;
                }
            }
        }
        Ok((self, info))
    }

    /// Returns true if the game is awaiting a verifier answer.
    pub fn is_awaiting_result(&self) -> bool {
        self.currently_chosen_verifier_option.is_some()
    }

    /// Return all possible moves. Notably these are not verified in every way:
    /// - Verifiers may return impossible results, leading to no solution.
    /// - Codes or verifiers may be chosen that do not provide information to
    ///   the player.
    pub fn possible_moves(&self) -> impl Iterator<Item = Move> + '_ {
        // This function looks messy to avoid allocating a Vec with moves.

        // If awaiting result, return both results
        [
            Move::VerifierSolution(VerifierSolution::Check),
            Move::VerifierSolution(VerifierSolution::Cross),
        ]
        .iter()
        .copied()
        .filter(|_| self.is_awaiting_result())
        // Otherwise,
        .chain(
            // Otherwise, if a code is chosen, choose a verifier
            (0..self.game.verifier_count())
                .map(|i| Move::ChooseVerifierOption(ChosenVerifierOption(i as u8)))
                .filter(|_| self.currently_selected_code.is_some())
                .chain(
                    // If the code was used once, or if no code was selected, choose new code
                    CodeSet::all()
                        .iter_bit_code()
                        .map(Move::ChooseNewCode)
                        .filter(|_| {
                            self.currently_selected_code.is_none()
                                || self.guessed_one_verifier_for_code
                        }),
                )
                .filter(|_| !self.is_awaiting_result()),
        )
    }

    fn is_maximizing_score(self) -> bool {
        !self.is_awaiting_result()
    }

    /// Perform minmax with alpha-beta pruning.
    fn alphabeta(self, mut alpha: StateScore, mut beta: StateScore) -> (StateScore, Option<Move>) {
        // If the game is solved, return the result.
        if self.is_solved() {
            (
                StateScore::solution(self.codes_guessed, self.verifiers_checked),
                None,
            )
        } else if self.is_maximizing_score() {
            let mut highest_score = StateScore::min();
            let mut best_move = None;
            for move_to_do in self.possible_moves() {
                let next_node = self.after_move(move_to_do);
                let score = match next_node {
                    Err(AfterMoveError::NoCodesLeft) => StateScore::no_solution(),
                    Err(AfterMoveError::InvalidMoveError) => panic!("invalid move!"),
                    Ok((_, Some(AfterMoveInfo::UselessVerifierCheck))) => StateScore::useless_verifier_check(),
                    Ok((state, None)) => state.alphabeta(alpha, beta).0,
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
                    Err(AfterMoveError::NoCodesLeft) => StateScore::no_solution(),
                    Err(AfterMoveError::InvalidMoveError) => panic!("invalid move"),
                    Ok((_, Some(AfterMoveInfo::UselessVerifierCheck))) => StateScore::useless_verifier_check(),
                    Ok((state, None)) => state.alphabeta(alpha, beta).0,
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
        let alpha = StateScore::min();
        // The worst possible game.
        let beta = StateScore::max();
        if let (score, Some(move_to_do)) = self.alphabeta(alpha, beta) {
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
