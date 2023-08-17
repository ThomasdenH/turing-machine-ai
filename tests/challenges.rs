use std::error::Error;

use turing_machine_ai::{game::Game, gametree::{State, Move, VerifierSolution::*}, code::{CodeError, Code}};

#[test]
fn challenge_b52_o00_b() -> Result<(), Box<dyn Error>> {
    let game = Game::new_from_verifier_numbers([12, 16, 18, 19, 21].iter().copied());
    let state = game.starting_state();

    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseNewCode(Code::from_digits(2, 1, 1)?));
    let (state, _) = state.after_move(move_to_do)?;

    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseVerifierOption(0.into()));
    let (state, _) = state.after_move(move_to_do)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Cross))?;

    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseVerifierOption(2.into()));
    let (state, _) = state.after_move(move_to_do)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Check))?;

    assert_eq!(state.solution(), Some(Code::from_digits(3, 3, 4)?));
    Ok(())
}