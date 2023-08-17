use turing_machine_ai::{game::Game, gametree::{State, Move, AfterMoveError}, code::Code};
use turing_machine_ai::gametree::VerifierSolution::*;

#[test]
fn test_01() -> Result<(), AfterMoveError> {
    let game = Game::new_from_verifier_numbers([4, 9, 11, 14].iter().copied());
    let state = State::new(&game);
    let (game_score, next_move) = state.find_best_move();
    assert_eq!(game_score.codes_guessed, 1);
    assert_eq!(game_score.verifiers_checked, 1);
    assert_eq!(next_move, Move::ChooseNewCode(Code::from_digits(1, 1, 1).unwrap()));

    let (state, _) = state.after_move(next_move)?;
    let (game_score, next_move) = state.find_best_move();
    assert_eq!(game_score.codes_guessed, 1);
    assert_eq!(game_score.verifiers_checked, 1);
    assert_eq!(next_move, Move::ChooseVerifier(0.into()));
    
    let (state, _) = state.after_move(next_move)?;
    assert!(state.is_awaiting_result());
    
    let (state, _) = state.after_move(Move::VerifierSolution(Cross))?;
    assert!(state.is_solved());
    assert_eq!(state.possible_codes().iter().next(), Some(Code::from_digits(2, 4, 1).unwrap()));

    Ok(())
}

#[test]
fn test_02() -> Result<(), AfterMoveError> {
    let game = Game::new_from_verifier_numbers([3, 7, 10, 14].iter().copied());
    let state = State::new(&game);
    let (_, next_move) = state.find_best_move();
    assert_eq!(next_move, Move::ChooseNewCode(Code::from_digits(2, 2, 1).unwrap()));

    let (state, _) = state.after_move(next_move)?;
    let (_, next_move) = state.find_best_move();
    assert_eq!(next_move, Move::ChooseVerifier(0.into()));

    let (state, _) = state.after_move(next_move)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Cross))?;
    let (_, next_move) = state.find_best_move();
    assert_eq!(next_move, Move::ChooseVerifier(1.into()));

    let (state, _) = state.after_move(next_move)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Check))?;
    let(_, next_move) = state.find_best_move();
    assert_eq!(next_move, Move::ChooseVerifier(3.into()));

    let (state, _) = state.after_move(next_move)?;
    let (state, _) = state.after_move(Move::VerifierSolution(Cross))?;
    assert!(state.is_solved());
    assert_eq!(state.possible_codes().iter().next(), Some(Code::from_digits(4, 3, 5).unwrap()));

    Ok(())
}
