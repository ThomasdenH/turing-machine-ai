use std::io::stdin;

use turing_machine_ai::game::Game;
use turing_machine_ai::gametree::{
    self, AfterMoveError, AfterMoveInfo, Move, State, VerifierSolution,
};

fn main() {
    let game = Game::new_from_verifier_numbers([3, 7, 10, 14].iter().copied());
    let possible_solutions = game.possible_solutions();
    let uniquely_satisfied = game.all_unique_satisfied_options();

    let mut state = State::new(&game, (&possible_solutions).into(), &uniquely_satisfied);
    while !state.is_solved() {
        println!(
            "There are still {} possible codes.",
            state.possible_solutions().size()
        );
        if state.is_awaiting_result() {
            loop {
                println!("What does the verifier tell you? x/v");
                let mut string = String::new();
                stdin().read_line(&mut string).unwrap();
                let state_result = match string.trim() {
                    "x" | "X" => state.after_move(Move::VerifierSolution(VerifierSolution::Cross)),
                    "v" | "V" => state.after_move(Move::VerifierSolution(VerifierSolution::Check)),
                    other => {
                        println!("Unknown selection: '{other}'");
                        continue;
                    }
                };
                match state_result {
                    Err(AfterMoveError::InvalidMoveError) => panic!("Invalid move!"),
                    Err(AfterMoveError::NoCodesLeft) => panic!("No codes left!"),
                    Ok((new_state, None)) => {
                        state = new_state;
                        break;
                    }
                    Ok((_, Some(AfterMoveInfo::UselessVerifierCheck))) => {
                        panic!("Useless verifier check")
                    }
                }
            }
        } else {
            let (score, move_to_do) = state.find_best_move();
            println!(
                "You will find the solution in {} codes and {} verifier checks.",
                score.codes_guessed, score.verifiers_checked
            );
            match state.after_move(move_to_do) {
                Err(AfterMoveError::NoCodesLeft) => {
                    println!("There are no possible codes left.");
                }
                Err(AfterMoveError::InvalidMoveError) => {
                    panic!("Invalid move!");
                }
                Ok((new_state, None)) => {
                    match move_to_do {
                        gametree::Move::ChooseNewCode(code) => println!("Choose code {code:?}."),
                        gametree::Move::ChooseVerifier(option) => {
                            println!("Choose verifier {option:?}.")
                        }
                        gametree::Move::VerifierSolution(_) => panic!(),
                    }
                    state = new_state;
                }
                Ok((_, Some(AfterMoveInfo::UselessVerifierCheck))) => {
                    println!("The chosen verifier does not give any new information.");
                }
            }
        }
    }

    println!(
        "Solved! Solution: {:?}",
        state.possible_solutions().possible_codes().next().unwrap()
    );
}
