use turing_machine_ai::{game::Game, verifier::get_verifier_by_number, gametree::{State, DoMoveResult, Move}, code::Code};

fn expect_state(do_move: DoMoveResult<'_>) -> State<'_> {
    match do_move {
        DoMoveResult::State(state) => state,
        _ => panic!("Unexpected DoMoveResult")
    }
}

#[test]
fn test_01() {
    let game = Game::new_from_verifiers(vec![
        get_verifier_by_number(4),
        get_verifier_by_number(9),
        get_verifier_by_number(11),
        get_verifier_by_number(14),
    ]);
    let state = State::new(&game);
    let (game_score, next_move) = state.find_best_move();
    assert_eq!(game_score.codes_guessed, 1);
    assert_eq!(game_score.verifiers_checked, 1);
    assert_eq!(next_move, Move::ChooseNewCode(Code::from_digits(1, 1, 1).into()));

    let state = expect_state(state.after_move(next_move));
    let (game_score, next_move) = state.find_best_move();
    assert_eq!(game_score.codes_guessed, 1);
    assert_eq!(game_score.verifiers_checked, 1);
    assert_eq!(next_move, Move::ChooseVerifierOption(0.into()));
    
    let state = expect_state(state.after_move(next_move));
    assert!(state.is_awaiting_result());
    
    let state = expect_state(state.after_move(Move::VerifierSolution(turing_machine_ai::gametree::VerifierSolution::Cross)));
    assert!(state.is_solved());
    assert_eq!(state.possible_codes().iter().next(), Some(Code::from_digits(2, 4, 1)));
}
