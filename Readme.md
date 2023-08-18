# Turing Machine AI

An AI for the Turing Machine board game!

Currently, you can
- Easily construct games from its verifiers.
- Do all deductions: find all possible verifier combinations and codes
- Find the best sequence of moves that will get you to the solution the quickest.

## Example
The following example solves the first game in the booklet:

```rust
use std::error::Error;
use turing_machine_ai::{
    game::Game,
    code::Code,
    gametree::{Move, State, AfterMoveError, VerifierSolution::*}
};

/// Solve the first puzzle from the booklet.
fn main() -> Result<(), Box<dyn Error>> {
    let game = Game::new_from_verifier_numbers([4, 9, 11, 14].iter().copied());
    let state = State::new(&game);
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
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
