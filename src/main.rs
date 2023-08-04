use turing_machine_ai::code::CodeSet;
use turing_machine_ai::game::Game;
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

    println!(
        "{:?}",
        game.all_assignments()
            .filter(|assignment| game.is_possible_solution(assignment))
            .map(|assignment| game.possible_codes_for_assignment(&assignment))
            .fold(CodeSet::empty(), |all, new| all.union_with(new))
    );
}
