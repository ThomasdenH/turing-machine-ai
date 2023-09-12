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
    code::Code,
    game::{ChosenVerifier, Game, PossibleSolutionFilter, SatisfiedOptions},
};

/// A struct representing the current game state.
///
/// It contains the possible
/// solutions for the verifier selection, the currently selected code (if any),
/// the currently selected verifier (if any), whether a verifier was tested, as
/// well as how many tests were performed.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct State<'a> {
    game: &'a Game,
    /// All the codes that are still possible solutions.
    possible_codes: PossibleSolutionFilter<'a>,
    current_selection: CodeVerifierChoice,
    codes_with_unique_assignment: &'a Vec<SatisfiedOptions>,
    codes_guessed_verifiers_checked: StateScore,
}

/// Indicates whether a code and a verifier have been selected.
#[derive(Eq, Clone, Copy, Debug, Hash, PartialEq)]
enum CodeVerifierChoice {
    /// Neither a code, nor a verifier has been selected.
    None,
    /// A code has been selected, but no verifier yet.
    /// The second argument indicates how many verifiers have been checked for this code.
    Code(SatisfiedOptions, u8),
    /// Both a code as well as a verifier have been selected.
    /// The second argument indicates how many verifiers have been checked for this code.
    CodeAndVerifier(SatisfiedOptions, u8, ChosenVerifier),
}

impl InternalMove {
    pub fn choose_code(code: Code, game: &Game) -> Self {
        Self::ChooseNewCode(SatisfiedOptions::for_code(code, game))
    }

    fn from_move(move_to_do: Move, game: &Game) -> Self {
        match move_to_do {
            Move::ChooseNewCode(code) => Self::choose_code(code, game),
            Move::ChooseVerifier(verifier) => Self::ChooseVerifier(verifier),
            Move::VerifierSolution(solution) => Self::VerifierSolution(solution),
        }
    }
}

/// A struct representing a score associated with a game state. The score
/// represents how desirable it is for the guesser. A perfect score compares
/// the highest whereas a game without a solution is the lowest possible score.
///
/// Internally, the score is inverted so that a perfect game is represented by a 0.
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
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
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
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

    fn add_verifier_check(&mut self) {
        self.0 += 1;
    }
    fn add_code_check(&mut self) {
        self.0 += 1 << 8;
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

/// A move to be taken for a particular game state. Choosing a code is
/// represented by its unique assignment.
#[derive(Eq, PartialEq, Clone, Copy, Debug, Hash)]
enum InternalMove {
    /// Choose a new code. This can not be played directly after
    /// [`Move::ChooseVerifier`] since we expect a verifier response using the
    /// variant [`Move::VerifierSolution`].
    ChooseNewCode(SatisfiedOptions),
    /// Provide a verifier response. This can only be played after
    /// [`Move::ChooseVerifier`].
    VerifierSolution(VerifierSolution),
    /// Choose a verifier. This cannot be played twice in a row, since we expect
    /// a verifier answer inbetween using [`Move::VerifierSolution`].
    ChooseVerifier(ChosenVerifier),
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

impl Move {
    fn from_internal_move(value: InternalMove, game: &Game) -> Self {
        match value {
            InternalMove::ChooseVerifier(verifier) => Move::ChooseVerifier(verifier),
            InternalMove::VerifierSolution(solution) => Move::VerifierSolution(solution),
            InternalMove::ChooseNewCode(satisfied) => Move::ChooseNewCode(satisfied.to_code(game)),
        }
    }
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
    pub fn new(
        game: &'a Game,
        possible_codes: PossibleSolutionFilter<'a>,
        all_unique_satisfied_options: &'a Vec<SatisfiedOptions>,
    ) -> Self {
        State {
            game,
            possible_codes,
            current_selection: CodeVerifierChoice::None,
            codes_guessed_verifiers_checked: StateScore(0),
            codes_with_unique_assignment: all_unique_satisfied_options,
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
        self,
        move_to_do: Move,
    ) -> Result<(State<'a>, Option<AfterMoveInfo>), AfterMoveError> {
        let move_internal = InternalMove::from_move(move_to_do, self.game);
        self.after_move_internal(move_internal)
    }

    fn after_move_internal(
        mut self,
        move_to_do: InternalMove,
    ) -> Result<(State<'a>, Option<AfterMoveInfo>), AfterMoveError> {
        use CodeVerifierChoice::*;
        use InternalMove::*;
        let mut info: Option<AfterMoveInfo> = Option::None;
        match (move_to_do, self.current_selection) {
            // Choosing a new code can be done if not waiting for a verifier response.
            (ChooseNewCode(code), Code(_, _) | None) => {
                self.current_selection = Code(code, 0);
                self.codes_guessed_verifiers_checked.add_code_check();
            }
            // Choosing a new verifier can be done if not waiting for a verifier response,
            // given a code was chosen.
            (ChooseVerifier(verifier), Code(code, verifiers_checked_for_this_code)) => {
                self.current_selection =
                    CodeAndVerifier(code, verifiers_checked_for_this_code + 1, verifier);
                self.codes_guessed_verifiers_checked.add_verifier_check();
            }
            // A verifier solution can be provided if a code and verifier have been selected.
            (
                VerifierSolution(solution),
                CodeAndVerifier(code, verifiers_checked_for_this_code, verifier),
            ) => {
                // Get all codes that correspond to a verifier option giving the provided answer.
                let new_possible_solutions = self
                    .possible_codes
                    .filter_through_verifier_check(verifier, code, solution);
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
    fn possible_moves(&self) -> impl Iterator<Item = InternalMove> + '_ {
        match self.current_selection {
            CodeVerifierChoice::CodeAndVerifier(_, _, _) => [
                InternalMove::VerifierSolution(VerifierSolution::Check),
                InternalMove::VerifierSolution(VerifierSolution::Cross),
            ]
            .iter()
            .copied(),
            CodeVerifierChoice::None => self
                .codes_with_unique_assignment
                .iter()
                .copied()
                .map(InternalMove::ChooseNewCode),
            CodeVerifierChoice::Code(_, verifiers_used_for_codes)
                if verifiers_used_for_codes != 0 =>
            {
                self.game
                    .iter_verifier_choices()
                    .map(InternalMove::ChooseVerifier)
                    .chain(
                        self.codes_with_unique_assignment
                            .iter()
                            .copied()
                            .map(InternalMove::ChooseNewCode),
                    )
            }
            CodeVerifierChoice::Code(_, _) => self
                .game
                .iter_verifier_choices()
                .map(InternalMove::ChooseVerifier),
        }
    }

    /// Returns whether the state demands maximizing the score. This
    /// corresponds to those states where the player must do a turn as opposed
    /// to waiting for a verifier answer.
    fn is_maximizing_score(self) -> bool {
        !self.is_awaiting_result()
    }

    /// Perform minmax with alpha-beta pruning.
    fn alphabeta(
        self,
        mut alpha: StateScore,
        mut beta: StateScore,
    ) -> (StateScore, Option<InternalMove>) {
        // Beta is the highest score that the player can definitely get.
        // Alpha is the lowest score that the player gets if unlucky.
        // If the game is solved, return the result.
        if self.is_solved() {
            (self.codes_guessed_verifiers_checked, None)
        } else if self.is_maximizing_score() {
            let mut highest_score = StateScore::min_score();
            let mut best_move = None;
            for move_to_do in self.possible_moves() {
                let next_node = self.after_move_internal(move_to_do);
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
                    // The worst case is definitely not this branch.
                    // To find the strategy that maximizes our minimum score, explore another branch.
                    break;
                }
                if score > alpha {
                    // We have found a way to increase our minimum score.
                    alpha = score;
                }
            }
            (highest_score, best_move)
        } else {
            let mut lowest_score = StateScore::max_score();
            for move_to_do in self.possible_moves() {
                let next_node = self.after_move_internal(move_to_do);
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
            (
                score.codes_and_verifiers_checked().unwrap(),
                Move::from_internal_move(move_to_do, self.game),
            )
        } else {
            panic!("No move possible");
        }
    }
}
