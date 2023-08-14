use turing_machine_ai::code::CodeSet;
use turing_machine_ai::game::Game;
use turing_machine_ai::gametree::{self, State, find_best_move};
use turing_machine_ai::verifier::get_verifier_by_number;

fn main() {
    let game = Game::new_from_verifiers(vec![
        get_verifier_by_number(13),
        get_verifier_by_number(16),
        get_verifier_by_number(23),
        get_verifier_by_number(33),
        get_verifier_by_number(34),
        get_verifier_by_number(45),
    ]);

    let possible_codes = game.all_assignments()
        .filter(|assignment| game.is_possible_solution(assignment))
        .map(|assignment| game.possible_codes_for_assignment(&assignment))
        .fold(CodeSet::empty(), |all, new| all.union_with(new));

    let mut state = State::new(&game, possible_codes);
    loop {
        println!(
            "{:?}",
            state.possible_codes()
        );
        if !state.is_awaiting_result() {
            let (state_score, move_to_do) = find_best_move(state);
            if let Some(move_to_do) = move_to_do {
                match state.after_move(move_to_do) {
                    gametree::DoMoveResult::NoCodesLeft => println!("No codes left"),
                    gametree::DoMoveResult::State(new_state) => {
                        println!("Do move {:?}", move_to_do);
                        state = new_state;
                    },
                    gametree::DoMoveResult::UselessVerifierCheck => println!("Useless verifier check")
                }
            } else {
                for (i, move_to_do) in state.possible_moves().iter().enumerate() {
                    println!("Possible move: {}: {:?}", i, move_to_do);
                }
                panic!("Don't know which move to take!");
            }
        } else {
            panic!("Awaiting result");
        }
    }
}
