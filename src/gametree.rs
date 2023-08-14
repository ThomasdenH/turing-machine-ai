use crate::{
    code::{BitCode, CodeSet},
    game::Game,
};

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

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
struct FoundSolution {
    codes_guessed: u8,
    verifiers_checked: u8,
    possible_codes: u8,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct StateScore(u16);

impl StateScore {
    fn no_solution() -> Self {
        StateScore(u16::MAX)
    }

    fn solution(codes: u8, verifier_checks: u8) -> Self {
        StateScore(u16::from(codes) << 8 | u16::from(verifier_checks))
    }

    fn useless_verifier_check() -> Self {
        StateScore(u16::MAX)
    }

    fn perfect_game() -> Self {
        StateScore(0)
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

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum VerifierSolution {
    Cross,
    Check,
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub struct ChosenVerifierOption(u8);

#[derive(Eq, PartialEq, Clone, Copy, Debug)]
pub enum Move {
    ChooseNewCode(BitCode),
    VerifierSolution(VerifierSolution),
    ChooseVerifierOption(ChosenVerifierOption),
}

impl Move {
    fn is_maximizing_score(&self) -> bool {
        if let Move::VerifierSolution(_) = self {
            false
        } else {
            true
        }
    }
}

impl<'a> State<'a> {
    pub fn new(game: &'a Game, possible_codes: CodeSet) -> Self {
        State {
            game,
            possible_codes,
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

    fn is_solved(self) -> bool {
        self.possible_codes.size() == 1
    }

    pub fn after_move(mut self, move_to_do: Move) -> DoMoveResult<'a> {
        match move_to_do {
            Move::ChooseNewCode(code) => {
                self.currently_selected_code = Some(code);
                self.codes_guessed += 1;
            }
            Move::ChooseVerifierOption(choose_verifier_option) => {
                debug_assert!(self.currently_chosen_verifier_option.is_none());
                self.currently_chosen_verifier_option = Some(choose_verifier_option);
                self.verifiers_checked += 1;
            }
            Move::VerifierSolution(verifier_solution) => {
                // Eliminate codes
                let mut bitmask_for_solution = CodeSet::empty();
                for code_set in self
                    .game
                    .verfier(self.currently_chosen_verifier_option.unwrap().0)
                    .options()
                {
                    let would_give_check = code_set
                        .code_set()
                        .contains_bit_code(self.currently_selected_code.unwrap());
                    let gives_check = verifier_solution == VerifierSolution::Check;
                    if would_give_check == gives_check {
                        bitmask_for_solution = bitmask_for_solution.union_with(code_set.code_set());
                    }
                }
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

    pub fn is_awaiting_result(&self) -> bool {
        self.currently_chosen_verifier_option.is_some()
    }

    pub fn possible_moves(&self) -> Vec<Move> {
        let mut moves = Vec::new();

        if self.is_awaiting_result() {
            // Already chosen a verifier option, resolve
            moves.append(&mut vec![
                Move::VerifierSolution(VerifierSolution::Check),
                Move::VerifierSolution(VerifierSolution::Cross),
            ]);
        } else {
            if let Some(code) = self.currently_selected_code {
                moves.append(
                    &mut (0..self.game.verifier_count())
                        .map(|i| Move::ChooseVerifierOption(ChosenVerifierOption(i as u8)))
                        .collect(),
                );
            }
            if self.currently_selected_code.is_none() || self.guessed_one_verifier_for_code {
                moves.append(
                    &mut CodeSet::all()
                        .iter_bit_code()
                        .map(|bit_code| Move::ChooseNewCode(bit_code))
                        .collect::<Vec<Move>>(),
                );
            }
        }
        moves
    }
}

pub fn find_best_move<'a>(node: State<'a>) -> (StateScore, Option<Move>) {
    // The optimal possible game.
    let alpha = StateScore::perfect_game();
    let beta = StateScore::no_solution();
    alphabeta(node, 0, alpha, beta, true)
}

pub fn alphabeta<'a>(
    node: State<'a>,
    depth: u32,
    mut alpha: StateScore,
    mut beta: StateScore,
    is_maximizing_score: bool,
) -> (StateScore, Option<Move>) {
    if node.is_solved() {
        (StateScore::solution(node.codes_guessed, node.verifiers_checked), None)
    } else if is_maximizing_score {
        let mut highest_score = StateScore::no_solution();
        let mut best_move = None;
        for move_to_do in node.possible_moves() {
            let next_node = node.after_move(move_to_do);
            let score = match next_node {
                DoMoveResult::NoCodesLeft => StateScore::no_solution(),
                DoMoveResult::UselessVerifierCheck => StateScore::useless_verifier_check(),
                DoMoveResult::State(state) => alphabeta(
                    state,
                    depth + 1,
                    alpha,
                    beta,
                    move_to_do.is_maximizing_score(),
                ).0,
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
        let mut lowest_score = StateScore::perfect_game();
        for move_to_do in node.possible_moves() {
            let next_node = node.after_move(move_to_do);
            let score = match next_node {
                DoMoveResult::NoCodesLeft => StateScore::no_solution(),
                DoMoveResult::UselessVerifierCheck => StateScore::useless_verifier_check(),
                DoMoveResult::State(state) => alphabeta(
                    state,
                    depth + 1,
                    alpha,
                    beta,
                    move_to_do.is_maximizing_score(),
                ).0,
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
