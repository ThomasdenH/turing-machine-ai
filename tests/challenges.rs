use std::error::Error;

use turing_machine_ai::{
    code::Code,
    game::Game,
    gametree::{Move, State, VerifierSolution::*},
};

#[test]
fn challenge_b52_o00_b() -> Result<(), Box<dyn Error>> {
    let game = Game::new_from_verifier_numbers([12, 16, 18, 19, 21].iter().copied());
    let possible_solutions = game.possible_solutions();
    let uniquely_satisfied = game.all_unique_satisfied_options();
    let state = State::new(&game, (&possible_solutions).into(), &uniquely_satisfied);

    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseNewCode(Code::from_digits(2, 1, 1)?));
    let (state, _) = state.after_move(move_to_do)?;

    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseVerifier(0.into()));
    let (state, _) = state.after_move(move_to_do)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Cross))?;

    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseVerifier(2.into()));
    let (state, _) = state.after_move(move_to_do)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Check))?;

    assert_eq!(state.solution(), Some(Code::from_digits(3, 3, 4)?));
    Ok(())
}

#[test]
fn challenge_c4d_ck4() -> Result<(), Box<dyn Error>> {
    let game = Game::new_from_verifier_numbers([18, 21, 37, 48].iter().copied());
    let possible_solutions = game.possible_solutions();
    let uniquely_satisfied = game.all_unique_satisfied_options();
    let state = State::new(&game, (&possible_solutions).into(), &uniquely_satisfied);

    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseNewCode(Code::from_digits(2, 1, 1)?));
    let (state, _) = state.after_move(move_to_do)?;
    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseVerifier(3.into()));
    let (state, _) = state.after_move(move_to_do)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Cross))?;

    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseNewCode(Code::from_digits(2, 3, 1)?));
    let (state, _) = state.after_move(move_to_do)?;
    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseVerifier(2.into()));
    let (state, _) = state.after_move(move_to_do)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Cross))?;

    let (_, move_to_do) = state.find_best_move();
    assert_eq!(move_to_do, Move::ChooseVerifier(3.into()));
    let (state, _) = state.after_move(move_to_do)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Check))?;

    assert_eq!(state.solution(), Some(Code::from_digits(1, 3, 5)?));

    Ok(())
}
