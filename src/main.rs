use std::io::stdin;

use turing_machine_ai::game::Game;
use turing_machine_ai::gametree::{self, Move, State, VerifierSolution};

fn main() {
    let game = Game::new_from_verifier_numbers([2, 16, 25, 33, 36, 44].iter().copied());

    let mut state = State::new(&game);
    while !state.is_solved() {
        println!("Possible codes:\n{:?}", state.possible_codes());
        if !state.is_awaiting_result() {
            let (score, move_to_do) = state.find_best_move();
            println!(
                "You will find the solution in {} codes and {} verifier checks.",
                score.codes_guessed, score.verifiers_checked
            );
            match state.after_move(move_to_do) {
                gametree::DoMoveResult::NoCodesLeft => {
                    println!("There are no possible codes left.")
                }
                gametree::DoMoveResult::State(new_state) => {
                    match move_to_do {
                        gametree::Move::ChooseNewCode(code) => println!("Choose code {:?}.", code),
                        gametree::Move::ChooseVerifierOption(option) => {
                            println!("Choose verifier {:?}.", option)
                        }
                        gametree::Move::VerifierSolution(_) => panic!(),
                    }
                    state = new_state;
                }
                gametree::DoMoveResult::UselessVerifierCheck => {
                    println!("The chosen verifier does not give any new information.")
                }
            }
        } else {
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
                    gametree::DoMoveResult::NoCodesLeft => panic!("No codes left!"),
                    gametree::DoMoveResult::State(new_state) => {
                        state = new_state;
                        break;
                    }
                    gametree::DoMoveResult::UselessVerifierCheck => {
                        panic!("Useless verifier check")
                    }
                }
            }
        }
    }

    println!("Solved! Solution: {:?}", state.possible_codes().iter().next().unwrap());
}
