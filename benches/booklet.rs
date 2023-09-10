use criterion::*;
use turing_machine_ai::{
    game::Game,
    gametree::{GameScore, Move, State},
};

fn bench(c: &mut Criterion) {
    let mut group = c.benchmark_group("booklet challenges");

    let booklet: [&[usize]; 20] = [
        &[4, 9, 11, 14],
        &[3, 7, 10, 14],
        &[4, 9, 13, 17],
        &[3, 8, 15, 16],
        &[2, 6, 14, 17],
        &[2, 7, 10, 13],
        &[8, 12, 15, 17],
        &[3, 5, 9, 15, 16],
        &[1, 7, 10, 12, 17],
        &[2, 6, 8, 12, 15],
        &[5, 10, 11, 15, 17],
        &[4, 9, 18, 20],
        &[11, 16, 19, 21],
        &[2, 13, 17, 20],
        &[5, 14, 18, 19, 20],
        &[2, 7, 12, 16, 19, 22],
        &[21, 31, 37, 39],
        &[23, 28, 41, 48],
        &[19, 24, 30, 31, 38],
        &[11, 22, 30, 33, 34, 40],
    ];

    for (index, challenge) in booklet.iter().enumerate() {
        let challenge_number = index + 1;
        if challenge_number == 2 || challenge_number > 5 {
            continue;
        }
        group.bench_function(format!("challenge {challenge_number}"), |b| {
            b.iter(|| find_best_first_move_for_game(challenge))
        });
    }
}

fn find_best_first_move_for_game(challenge: &[usize]) -> Option<(GameScore, Move)> {
    let game = Game::new_from_verifier_numbers(challenge.iter().copied());
    let solutions = &game.possible_solutions();
    let state = State::new(&game, solutions.into());
    if !state.is_solved() {
        Some(state.find_best_move())
    } else {
        None
    }
}

criterion_group!(benches, bench);
criterion_main!(benches);
