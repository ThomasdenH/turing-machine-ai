//! Contains all stateful logic for the game.
//!
//! This module contains the tools to find the best course of action for
//! solving a particular game.

// TODO: There is a subltle bug in this code. Even if a verifier check doesn't
// eliminate codes immediately, it may still be usefull because the elimination of verifier
// options itself may be useful. This means that we prune out these branches too
// quickly. Furthermore, we probably have to store possible verifier options
// instead of/in addition to possible codes.
//
// Example: Suppose we know it's two possible codes:
// △ □ ○
// 3 5 1
// 1 5 3
//
// and we need information from one verfier: verifier 48.
//  △ < □   △ < ○   □ < ○
//  △ = □   △ = ○   □ = ○
//  △ > □   △ > ○   □ > ○
//
// Suppose we test 4 5 5, which gives a Check. Then no code can be eliminated on
// the face of it, since the criterion may be △ < □, which is true in both
// cases. However, we know that the four verifiers are sufficient and so that
// the criterion must have been △ < ○, eliminating code 3 5 1.

use std::fmt::Debug;

use auto_enums::auto_enum;
use thiserror::Error;

use crate::{
    code::{Code, Set},
    game::{ChosenVerifier, Game, PossibleSolutionFilter},
};

/// A struct representing the current game state.
///
/// It contains the possible
/// solutions for the verifier selection, the currently selected code (if any),
/// the currently selected verifier (if any), whether a verifier was tested, as
/// well as how many tests were performed.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct State<'a> {
    game: &'a Game,
    /// All the codes that are still possible solutions.
    possible_codes: PossibleSolutionFilter<'a>,
    current_selection: CodeVerifierChoice,
    codes_guessed: u8,
    verifiers_checked: u8,
}

/// Indicates whether a code and a verifier have been selected.
#[derive(Eq, Clone, Copy, Debug, Hash, PartialEq)]
enum CodeVerifierChoice {
    /// Neither a code, nor a verifier has been selected.
    None,
    /// A code has been selected, but no verifier yet.
    /// The second argument indicates how many verifiers have been checked for this code.
    Code(Code, u8),
    /// Both a code as well as a verifier have been selected.
    /// The second argument indicates how many verifiers have been checked for this code.
    CodeAndVerifier(Code, u8, ChosenVerifier),
}

/// A struct representing a score associated with a game state. The score
/// represents how desirable it is for the guesser. A perfect score compares
/// the highest whereas a game without a solution is the lowest possible score.
///
/// Internally, the score is inverted so that a perfect game is represented by a 0.
#[derive(Copy, Clone, Eq, PartialEq)]
struct StateScore(u16);

impl Debug for StateScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(game_score) = self.codes_and_verifiers_checked() {
            write!(f, "{game_score:?}")
        } else {
            write!(f, "useless verifier check")
        }
    }
}

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

    /// This is represented by the worst possible outcome for the player.
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
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
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
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum VerifierSolution {
    Cross,
    Check,
}

/// A move to be taken for a particular game state.
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
pub enum Move {
    /// Choose a new code. This can not be played directly after
    /// [`Move::ChooseVerifier`] since we expect a verifier response using the
    /// variant [`Move::VerifierSolution`].
    ChooseNewCode(Code),
    /// Provide a verifier response. This can only be played after
    /// [`Move::ChooseVerifier`].
    VerifierSolution(VerifierSolution),
    /// Choose a verifier. This cannot be played twice in a row, since we expect
    /// a verifier answer inbetween using [`Move::VerifierSolution`].
    ChooseVerifier(ChosenVerifier),
}

/// An error which may be returned by [`State::after_move`].
#[derive(Copy, Clone, Eq, PartialEq, Error, Debug, Hash)]
pub enum AfterMoveError {
    /// This variant indicates that the move was invalid. This may occur for
    /// example when a new code is chosen when the state was waiting for a
    /// verifier response.
    #[error("invalid move for game state")]
    InvalidMoveError,
    /// This variant indicates that there is no solution left for the resulting
    /// game state. This means that a wrong verifier answer was provided or
    /// that the game was invalid to begin with.
    #[error("there are no solutions left for this game state")]
    NoCodesLeft,
}

impl<'a> State<'a> {
    #[must_use]
    pub fn new(game: &'a Game, possible_codes: PossibleSolutionFilter<'a>) -> Self {
        State {
            game,
            possible_codes,
            current_selection: CodeVerifierChoice::None,
            codes_guessed: 0,
            verifiers_checked: 0,
        }
    }

    /// Get all possible codes for this game state.
    #[must_use]
    pub fn possible_solutions(self) -> PossibleSolutionFilter<'a> {
        self.possible_codes
    }

    #[must_use]
    pub fn is_solved(self) -> bool {
        self.possible_codes.size() == 1
    }

    /// If solved, returns the solution. Otherwise, it returns `None`.
    #[must_use]
    pub fn solution(self) -> Option<Code> {
        self.possible_codes.solution()
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
        use CodeVerifierChoice::*;
        use Move::*;
        let mut info: Option<AfterMoveInfo> = Option::None;
        match (move_to_do, self.current_selection) {
            // Choosing a new code can be done if not waiting for a verifier response.
            (ChooseNewCode(code), Code(_, _) | None) => {
                self.current_selection = Code(code, 0);
                self.codes_guessed += 1;
            }
            // Choosing a new verifier can be done if not waiting for a verifier response,
            // given a code was chosen.
            (ChooseVerifier(verifier), Code(code, verifiers_checked_for_this_code)) => {
                self.current_selection =
                    CodeAndVerifier(code, verifiers_checked_for_this_code + 1, verifier);
                self.verifiers_checked += 1;
            }
            // A verifier solution can be provided if a code and verifier have been selected.
            (
                VerifierSolution(solution),
                CodeAndVerifier(code, verifiers_checked_for_this_code, verifier),
            ) => {
                // Get all codes that correspond to a verifier option giving the provided answer.
                let new_possible_solutions = self
                    .possible_codes
                    .filter_through_verifier_check(self.game, verifier, code, solution);
                if new_possible_solutions.size() == self.possible_codes.size() {
                    info = Some(AfterMoveInfo::UselessVerifierCheck);
                } else if new_possible_solutions.is_empty() {
                    return Err(AfterMoveError::NoCodesLeft);
                } else {
                    self.possible_codes = new_possible_solutions;
                }

                // If three verifiers were checked, we must select a new
                // code. Otherwise, reset just the verifier selection.
                if verifiers_checked_for_this_code == 3 {
                    self.current_selection = None;
                } else {
                    self.current_selection = Code(code, verifiers_checked_for_this_code);
                }
            }
            _ => return Err(AfterMoveError::InvalidMoveError),
        }
        Ok((self, info))
    }

    /// Returns true if the game is awaiting a verifier answer.
    #[must_use]
    pub fn is_awaiting_result(&self) -> bool {
        matches!(
            self.current_selection,
            CodeVerifierChoice::CodeAndVerifier(_, _, _)
        )
    }

    /// Returns true if a code was selected.
    #[must_use]
    pub fn has_selected_code(&self) -> bool {
        matches!(
            self.current_selection,
            CodeVerifierChoice::CodeAndVerifier(_, _, _) | CodeVerifierChoice::Code(_, _)
        )
    }

    /// Return all possible moves. Notably these are not verified in every way:
    /// - Verifiers may return impossible results, leading to no solution.
    /// - Codes or verifiers may be chosen that do not provide information to
    ///   the player.
    #[auto_enum(Iterator)]
    pub fn possible_moves(&self) -> impl Iterator<Item = Move> {
        match self.current_selection {
            CodeVerifierChoice::CodeAndVerifier(_, _, _) => {
                [
                    Move::VerifierSolution(VerifierSolution::Check),
                    Move::VerifierSolution(VerifierSolution::Cross),
                ]
                .iter()
                .copied()
            },
            CodeVerifierChoice::None => Set::all().into_iter().map(Move::ChooseNewCode),
            CodeVerifierChoice::Code(_, verifiers_used_for_codes) if verifiers_used_for_codes != 0 => {
                self.game
                    .iter_verifier_choices()
                    .map(Move::ChooseVerifier)
                    .chain(Set::all().into_iter().map(Move::ChooseNewCode))
            },
            CodeVerifierChoice::Code(_, _) => {
                self.game
                    .iter_verifier_choices()
                    .map(Move::ChooseVerifier)
            }
        }
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
    use super::StateScore;
    use crate::gametree::GameScore;

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
