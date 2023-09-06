use std::fmt::Debug;

use thiserror::Error;

use crate::{
    code::{Code, Set},
    game::{ChosenVerifier, Game},
    verifier::VerifierOption,
};

/// A struct representing the current game state. It contains the possible
/// solutions for the verifier selection, the currently selected code (if any),
/// the currently selected verifier (if any), whether a verifier was tested, as
/// well as how many tests were performed.
#[derive(Copy, Clone, Debug)]
pub struct State<'a> {
    game: &'a Game,
    /// All the codes that are still possible solutions.
    possible_codes: Set,
    currently_selected_code: Option<Code>,
    currently_chosen_verifier: Option<ChosenVerifier>,
    has_guessed_one_verifier_for_code: bool,
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

/// This represents the current "score" associated with the game state.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct GameScore {
    pub codes_guessed: u8,
    pub verifiers_checked: u8,
}

impl StateScore {
    fn no_solution() -> Self {
        // A state without a solution gives the best possible score, which is
        // the worst possible score for the "opponent". This
        // ensures that they will never pick this result (provided there is a solution).
        //
        // Sometimes there is only a single verifier option giving a valid
        // code, which would return [`StateScore::useless_verifier_check`].
        // This is always the better score for the opponent, making the verifier
        // check the worst possible action for the player.
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

    /// Get how many codes and verifiers were checked for this game score. If
    /// the game did not finish for whatever reason, this function will return
    /// `None`.
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

    fn min_score() -> Self {
        StateScore(u16::MAX)
    }

    fn max_score() -> Self {
        StateScore(u16::MIN)
    }
}

/// Additional info that may be returned by the function `State::after_move`.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub enum AfterMoveInfo {
    /// The move checked a verifier that did not provide additional
    /// information about the game solution.
    UselessVerifierCheck,
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

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Move {
    ChooseNewCode(Code),
    VerifierSolution(VerifierSolution),
    ChooseVerifier(ChosenVerifier),
}

#[derive(Copy, Clone, Eq, PartialEq, Error, Debug)]
pub enum AfterMoveError {
    #[error("invalid move for game state")]
    InvalidMoveError,
    #[error("there are no solutions left for this game state")]
    NoCodesLeft,
}

impl<'a> State<'a> {
    #[must_use]
    pub fn new(game: &'a Game) -> Self {
        State {
            game,
            possible_codes: game.possible_solutions(),
            currently_selected_code: None,
            currently_chosen_verifier: None,
            has_guessed_one_verifier_for_code: false,
            codes_guessed: 0,
            verifiers_checked: 0,
        }
    }

    pub fn possible_codes(self) -> Set {
        self.possible_codes
    }

    #[must_use]
    pub fn is_solved(self) -> bool {
        self.possible_codes.size() == 1
    }

    /// If solved, returns the solution. Otherwise, it returns `None`.
    #[must_use]
    pub fn solution(self) -> Option<Code> {
        self.possible_codes
            .into_iter()
            .next()
            .filter(|_| self.is_solved())
    }

    /// Return the state after performing the given move. If the given move is
    /// invalid, this function returns an error. In addition to the state
    /// itself, this function will sometimes provide additional information
    /// about the transition as the second argument of the tuple.
    /// 
    /// # Errors
    /// This function returns an [`AfterMoveError`] in one of two cases:
    /// - [`AfterMoveError::InvalidMoveError`] indicates that the provided move
    ///     was invalid. For example, a verifier was chosen while still waiting
    ///     on the result of another verifier.
    /// - [`AfterMoveError::NoCodesLeft`] indicates that the game state is
    ///     invalid. Either the provided game has no solution or one of the
    ///     verifiers was supplied with the wrong response.
    pub fn after_move(
        mut self,
        move_to_do: Move,
    ) -> Result<(State<'a>, Option<AfterMoveInfo>), AfterMoveError> {
        let mut info = None;
        match move_to_do {
            Move::ChooseNewCode(code) => {
                if self.is_awaiting_result() {
                    return Err(AfterMoveError::InvalidMoveError);
                }
                self.currently_selected_code = Some(code);
                self.codes_guessed += 1;
                self.has_guessed_one_verifier_for_code = false;
            }
            Move::ChooseVerifier(choose_verifier_option) => {
                if self.is_awaiting_result() {
                    return Err(AfterMoveError::InvalidMoveError);
                }
                self.currently_chosen_verifier = Some(choose_verifier_option);
                self.verifiers_checked += 1;
            }
            Move::VerifierSolution(verifier_solution) => {
                if let Some(chosen_verifier) = self.currently_chosen_verifier {
                    // Get all codes that correspond to a verifier option giving the provided answer.
                    let bitmask_for_solution = self
                        .game
                        .verfier(chosen_verifier)
                        .options()
                        .map(VerifierOption::code_set)
                        .filter(|code_set| {
                            let would_give_check =
                                code_set.contains(self.currently_selected_code.unwrap());
                            let gives_check = verifier_solution == VerifierSolution::Check;
                            would_give_check == gives_check
                        })
                        .collect::<Set>();
                    let possible_codes = self.possible_codes;
                    let new_possible_codes = possible_codes.intersected_with(bitmask_for_solution);
                    if new_possible_codes == possible_codes {
                        info = Some(AfterMoveInfo::UselessVerifierCheck);
                    } else if new_possible_codes.size() == 0 {
                        return Err(AfterMoveError::NoCodesLeft);
                    } else {
                        self.possible_codes = new_possible_codes;
                    }

                    self.currently_chosen_verifier = None;
                    if self.verifiers_checked == 3 {
                        self.currently_selected_code = None;
                    }
                } else {
                    return Err(AfterMoveError::InvalidMoveError);
                }
            }
        }
        Ok((self, info))
    }

    /// Returns true if the game is awaiting a verifier answer.
    #[must_use]
    pub fn is_awaiting_result(&self) -> bool {
        self.currently_chosen_verifier.is_some()
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
            self.game
                .iter_verifier_choices()
                .map(Move::ChooseVerifier)
                .filter(|_| self.currently_selected_code.is_some())
                .chain(
                    // If the code was used once, or if no code was selected, choose new code
                    Set::all().into_iter().map(Move::ChooseNewCode).filter(|_| {
                        self.currently_selected_code.is_none()
                            || self.has_guessed_one_verifier_for_code
                    }),
                )
                .filter(|_| !self.is_awaiting_result()),
        )
    }

    /// Returns whether the state demands maximizing the score. This
    /// corresponds to those states where the player must do a turn as opposed
    /// to waiting for a verifier answer.
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
            let mut highest_score = StateScore::min_score();
            let mut best_move = None;
            for move_to_do in self.possible_moves() {
                let next_node = self.after_move(move_to_do);
                let score = match next_node {
                    Err(AfterMoveError::NoCodesLeft) => StateScore::no_solution(),
                    Err(AfterMoveError::InvalidMoveError) => panic!("invalid move!"),
                    Ok((_, Some(AfterMoveInfo::UselessVerifierCheck))) => {
                        StateScore::useless_verifier_check()
                    }
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
            let mut lowest_score = StateScore::max_score();
            for move_to_do in self.possible_moves() {
                let next_node = self.after_move(move_to_do);
                let score = match next_node {
                    Err(AfterMoveError::NoCodesLeft) => StateScore::no_solution(),
                    Err(AfterMoveError::InvalidMoveError) => panic!("invalid move"),
                    Ok((_, Some(AfterMoveInfo::UselessVerifierCheck))) => {
                        StateScore::useless_verifier_check()
                    }
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
    ///
    /// # Panics
    /// This function will panic if the state is currently awaiting a verifier
    /// answer or if the game has already been solved.
    #[must_use]
    pub fn find_best_move(self) -> (GameScore, Move) {
        assert!(!self.is_awaiting_result() && !self.is_solved());
        // The optimal possible game.
        let alpha = StateScore::min_score();
        // The worst possible game.
        let beta = StateScore::max_score();
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
