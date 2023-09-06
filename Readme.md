[![crates.io](https://img.shields.io/crates/v/turing_machine_ai.svg)](https://crates.io/crates/turing-machine-ai)
[![docs.rs](https://img.shields.io/docsrs/turing_machine_ai)](https://docs.rs/turing-machine-ai/)

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

## Theory

This AI is based on the realisation that all deductions can be made at the start of the game, from the verifiers alone. As a first step then, a set is produced of all codes that uniquely correspond to a verifier outcome, and where no verifier is useless.

This gives a set of possible codes that normally small enough to exhaustively search for the best course of action. This is not a trivial search problem, since which verifier to check for a code may depend on the result of the previous check. As such a chosen code must work for multiple paths.

The search is an exhaustive min-max tree search with alpha-beta pruning, with some additional heuristics. Perhaps the most important restriction is that every check should narrow down the number of possible codes. If no verifier provides information, a new code should be picked instead of doing a useless verifier check. This gives a bound on the number of actions to take.

The code is made efficient by packing sets of codes into `u128` (there are 5*5*5 codes), which makes operations on sets very quick and heapless.

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
