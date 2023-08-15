# Turing Machine AI

An AI for the Turing Machine board game!

Currently, you can
- Easily construct games from its verifiers.
- Do all deductions: find all possible verifier combinations and codes
- Find the best sequence of moves that will get you to the solution the quickest.

## Example
The following example solves the first game in the booklet:

```rust
fn main() -> Result<(), AfterMoveError> {
    let game = Game::new_from_verifier_numbers([4, 9, 11, 14].iter().copied());
    let state = State::new(&game);
    let (game_score, next_move) = state.find_best_move();
    assert_eq!(game_score.codes_guessed, 1);
    assert_eq!(game_score.verifiers_checked, 1);
    assert_eq!(next_move, Move::ChooseNewCode(Code::from_digits(1, 1, 1).into()));

    let (state, _) = state.after_move(next_move)?;
    let (game_score, next_move) = state.find_best_move();
    assert_eq!(game_score.codes_guessed, 1);
    assert_eq!(game_score.verifiers_checked, 1);
    assert_eq!(next_move, Move::ChooseVerifierOption(0.into()));
    
    let (state, _) = state.after_move(next_move)?;
    assert!(state.is_awaiting_result());
    
    let (state, _) = state.after_move(Move::VerifierSolution(Cross))?;
    assert!(state.is_solved());
    assert_eq!(state.possible_codes().iter().next(), Some(Code::from_digits(2, 4, 1)));

    Ok(())
}
```
