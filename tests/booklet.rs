use std::error::Error;

use turing_machine_ai::gametree::VerifierSolution::*;
use turing_machine_ai::{
    code::Code,
    game::Game,
    gametree::{Move, State},
};

fn test_game(verifiers: &[usize], moves: &[Move], solution: Code) -> Result<(), Box<dyn Error>> {
    let game = Game::new_from_verifier_numbers(verifiers.iter().copied());
    let possible_solutions = game.possible_solutions();
    let mut state = State::new(&game, (&possible_solutions).into());

    let actual_codes_guessed = moves
        .iter()
        .filter(|expected_move| matches!(expected_move, Move::ChooseNewCode(_)))
        .count();
    let actual_verifiers_checked = moves
        .iter()
        .filter(|expected_move| matches!(expected_move, Move::ChooseVerifier(_)))
        .count();

    for next_move in moves {
        if !matches!(next_move, Move::VerifierSolution(_)) {
            let (game_score, best_next_move) = state.find_best_move();
            assert!(actual_codes_guessed == usize::from(game_score.codes_guessed));
            assert!(actual_verifiers_checked == usize::from(game_score.verifiers_checked));
            assert_eq!(*next_move, best_next_move);
        }
        state = state.after_move(*next_move)?.0;
    }

    assert!(state.is_solved());
    assert_eq!(state.solution(), Some(solution));
    Ok(())
}

#[test]
fn test_01() -> Result<(), Box<dyn Error>> {
    let game = Game::new_from_verifier_numbers([4, 9, 11, 14].iter().copied());
    let possible_solutions = game.possible_solutions();
    let state = State::new(&game, (&possible_solutions).into());
    let (game_score, next_move) = state.find_best_move();
    assert_eq!(game_score.codes_guessed, 1);
    assert_eq!(game_score.verifiers_checked, 1);
    assert_eq!(next_move, Move::ChooseNewCode(Code::from_digits(1, 1, 1)?));

    let (state, _) = state.after_move(next_move)?;
    let (game_score, next_move) = state.find_best_move();
    assert_eq!(game_score.codes_guessed, 1);
    assert_eq!(game_score.verifiers_checked, 1);
    assert_eq!(next_move, Move::ChooseVerifier(0.into()));

    let (state, _) = state.after_move(next_move)?;
    assert!(state.is_awaiting_result());

    let (state, _) = state.after_move(Move::VerifierSolution(Cross))?;
    assert!(state.is_solved());
    assert_eq!(state.solution(), Some(Code::from_digits(2, 4, 1)?));

    Ok(())
}

#[test]
fn test_02() -> Result<(), Box<dyn Error>> {
    let game = Game::new_from_verifier_numbers([3, 7, 10, 14].iter().copied());
    let possible_solutions = game.possible_solutions();
    let state = State::new(&game, (&possible_solutions).into());

    let (_, next_move) = state.find_best_move();
    assert_eq!(next_move, Move::ChooseNewCode(Code::from_digits(2, 3, 1)?));
    let (state, _) = state.after_move(next_move)?;

    let (_, next_move) = state.find_best_move();
    assert_eq!(next_move, Move::ChooseVerifier(1.into()));
    let (state, _) = state.after_move(next_move)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Check))?;

    let (_, next_move) = state.find_best_move();
    assert_eq!(next_move, Move::ChooseVerifier(3.into()));
    let (state, _) = state.after_move(next_move)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Cross))?;

    assert!(state.is_solved());
    assert_eq!(state.solution(), Some(Code::from_digits(4, 3, 5)?));
    Ok(())
}

#[test]
fn test_03() -> Result<(), Box<dyn Error>> {
    test_game(
        &[4, 9, 13, 17],
        &[
            Move::ChooseNewCode(Code::from_digits(1, 1, 1)?),
            Move::ChooseVerifier(2.into()),
            Move::VerifierSolution(Cross),
            Move::ChooseVerifier(3.into()),
            Move::VerifierSolution(Check),
        ],
        Code::from_digits(3, 3, 1)?,
    )
}

#[test]
fn test_04() -> Result<(), Box<dyn Error>> {
    test_game(
        &[3, 8, 15, 16],
        &[
            Move::ChooseNewCode(Code::from_digits(2, 1, 1)?),
            Move::ChooseVerifier(0.into()),
            Move::VerifierSolution(Cross),
            Move::ChooseVerifier(2.into()),
            Move::VerifierSolution(Cross),
        ],
        Code::from_digits(3, 4, 5)?,
    )
}

#[test]
fn test_05() -> Result<(), Box<dyn Error>> {
    test_game(
        &[2, 6, 14, 17],
        &[
            Move::ChooseNewCode(Code::from_digits(1, 1, 1)?),
            Move::ChooseVerifier(1.into()),
            Move::VerifierSolution(Check),
        ],
        Code::from_digits(3, 5, 4)?,
    )
}
