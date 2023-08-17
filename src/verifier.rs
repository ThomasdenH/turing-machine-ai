use std::fmt::Debug;

use arrayvec::ArrayVec;

use crate::code::{Code, CodeSet, Order};

/// Get a verifier by its (one-indexed) number in the game.
pub fn get_verifier_by_number(number: usize) -> Verifier {
    let verifiers: [Verifier; 48] = [
        // 1
        Verifier::from_description_and_options(
            "the △ number compared to 1",
            &[
                VerifierOption::from_description_and_closure("△ = 1", |code| {
                    code.triangle() == 1
                }),
                VerifierOption::from_description_and_closure("△ > 1", |code| code.triangle() > 1),
            ],
        ),
        // 2
        Verifier::from_description_and_options(
            "the △ number compared to 3",
            &[
                VerifierOption::from_description_and_closure("△ < 3", |code| code.triangle() < 3),
                VerifierOption::from_description_and_closure("△ = 3", |code| {
                    code.triangle() == 3
                }),
                VerifierOption::from_description_and_closure("△ > 3", |code| code.triangle() > 3),
            ],
        ),
        // 3
        Verifier::from_description_and_options(
            "the □ number compared to 3",
            &[
                VerifierOption::from_description_and_closure("□ < 3", |code| code.square() < 3),
                VerifierOption::from_description_and_closure("□ = 3", |code| code.square() == 3),
                VerifierOption::from_description_and_closure("□ > 3", |code| code.square() > 3),
            ],
        ),
        // 4
        Verifier::from_description_and_options(
            "the □ number compared to 4",
            &[
                VerifierOption::from_description_and_closure("□ < 4", |code| code.square() < 4),
                VerifierOption::from_description_and_closure("□ = 4", |code| code.square() == 4),
                VerifierOption::from_description_and_closure("□ > 4", |code| code.square() > 4),
            ],
        ),
        // 5
        Verifier::from_description_and_options(
            "if △ is even or odd",
            &[
                VerifierOption::from_description_and_closure("△ is even", |code| {
                    code.triangle() % 2 == 0
                }),
                VerifierOption::from_description_and_closure("△ is odd", |code| {
                    code.triangle() % 2 == 1
                }),
            ],
        ),
        // 6
        Verifier::from_description_and_options(
            "if □ is even or odd",
            &[
                VerifierOption::from_description_and_closure("□ is even", |code| {
                    code.square() % 2 == 0
                }),
                VerifierOption::from_description_and_closure("□ is odd", |code| {
                    code.square() % 2 == 1
                }),
            ],
        ),
        // 7
        Verifier::from_description_and_options(
            "if ○ is even or odd",
            &[
                VerifierOption::from_description_and_closure("○ is even", |code| {
                    code.circle() % 2 == 0
                }),
                VerifierOption::from_description_and_closure("○ is odd", |code| {
                    code.circle() % 2 == 1
                }),
            ],
        ),
        // 8
        Verifier::from_description_and_options(
            "the number of 1s in the code",
            &[
                VerifierOption::from_description_and_closure("zero 1s", |code| {
                    code.count_digit(1) == 0
                }),
                VerifierOption::from_description_and_closure("one 1", |code| {
                    code.count_digit(1) == 1
                }),
                VerifierOption::from_description_and_closure("two 1s", |code| {
                    code.count_digit(1) == 2
                }),
                VerifierOption::from_description_and_closure("three 1s", |code| {
                    code.count_digit(1) == 3
                }),
            ],
        ),
        // 9
        Verifier::from_description_and_options(
            "the number of 3s in the code",
            &[
                VerifierOption::from_description_and_closure("zero 3s", |code| {
                    code.count_digit(3) == 0
                }),
                VerifierOption::from_description_and_closure("one 3", |code| {
                    code.count_digit(3) == 1
                }),
                VerifierOption::from_description_and_closure("two 3s", |code| {
                    code.count_digit(3) == 2
                }),
                VerifierOption::from_description_and_closure("three 3s", |code| {
                    code.count_digit(3) == 3
                }),
            ],
        ),
        // 10
        Verifier::from_description_and_options(
            "the number of 4s in the code",
            &[
                VerifierOption::from_description_and_closure("zero 4s", |code| {
                    code.count_digit(4) == 0
                }),
                VerifierOption::from_description_and_closure("one 4", |code| {
                    code.count_digit(4) == 1
                }),
                VerifierOption::from_description_and_closure("two 4s", |code| {
                    code.count_digit(4) == 2
                }),
                VerifierOption::from_description_and_closure("three 4s", |code| {
                    code.count_digit(4) == 3
                }),
            ],
        ),
        // 11
        Verifier::from_description_and_options(
            "the △ number compared to the □ number",
            &[
                VerifierOption::from_description_and_closure("△ < □", |code| {
                    code.triangle() < code.square()
                }),
                VerifierOption::from_description_and_closure("△ = □", |code| {
                    code.triangle() == code.square()
                }),
                VerifierOption::from_description_and_closure("△ > □", |code| {
                    code.triangle() > code.square()
                }),
            ],
        ),
        // 12
        Verifier::from_description_and_options(
            "the △ number compared to the ○ number",
            &[
                VerifierOption::from_description_and_closure("△ < ○", |code| {
                    code.triangle() < code.circle()
                }),
                VerifierOption::from_description_and_closure("△ = ○", |code| {
                    code.triangle() == code.circle()
                }),
                VerifierOption::from_description_and_closure("△ > ○", |code| {
                    code.triangle() > code.circle()
                }),
            ],
        ),
        // 13
        Verifier::from_description_and_options(
            "the □ number compared to the ○ number",
            &[
                VerifierOption::from_description_and_closure("□ < ○", |code| {
                    code.square() < code.circle()
                }),
                VerifierOption::from_description_and_closure("□ = ○", |code| {
                    code.square() == code.circle()
                }),
                VerifierOption::from_description_and_closure("□ > ○", |code| {
                    code.square() > code.circle()
                }),
            ],
        ),
        // 14
        Verifier::from_description_and_options(
            "which colour's number is smaller than either of the others",
            &[
                VerifierOption::from_description_and_closure("△ < □, ○", |code| {
                    code.triangle() < code.square() && code.triangle() < code.circle()
                }),
                VerifierOption::from_description_and_closure("□ < △, ○", |code| {
                    code.square() < code.triangle() && code.square() < code.circle()
                }),
                VerifierOption::from_description_and_closure("○ < □, △", |code| {
                    code.circle() < code.square() && code.circle() < code.triangle()
                }),
            ],
        ),
        // 15
        Verifier::from_description_and_options(
            "which colour's number is larger than either of the others",
            &[
                VerifierOption::from_description_and_closure("△ > □, ○", |code| {
                    code.triangle() > code.square() && code.triangle() > code.circle()
                }),
                VerifierOption::from_description_and_closure("□ > △, ○", |code| {
                    code.square() > code.triangle() && code.square() > code.circle()
                }),
                VerifierOption::from_description_and_closure("○ > □, △", |code| {
                    code.circle() > code.square() && code.circle() > code.triangle()
                }),
            ],
        ),
        // 16
        Verifier::from_description_and_options(
            "the number of even numbers compared to the number of odd numbers",
            &[
                VerifierOption::from_description_and_closure("EVEN > ODD", |code| {
                    code.count_even() >= 2
                }),
                VerifierOption::from_description_and_closure("EVEN < ODD", |code| {
                    code.count_even() <= 1
                }),
            ],
        ),
        // 17
        Verifier::from_description_and_options(
            "how many even numbers there are in the code",
            &[
                VerifierOption::from_description_and_closure("zero even numbers", |code| {
                    code.count_even() == 0
                }),
                VerifierOption::from_description_and_closure("one even number", |code| {
                    code.count_even() == 1
                }),
                VerifierOption::from_description_and_closure("two even numbers", |code| {
                    code.count_even() == 2
                }),
                VerifierOption::from_description_and_closure("three even numbers", |code| {
                    code.count_even() == 3
                }),
            ],
        ),
        // 18
        Verifier::from_description_and_options(
            "if the sum of all the numbers is even or odd",
            &[
                VerifierOption::from_description_and_closure("△ + □ + ○ = EVEN", |code| {
                    (code.triangle() + code.square() + code.circle()) % 2 == 0
                }),
                VerifierOption::from_description_and_closure("△ + □ + ○ = ODD", |code| {
                    (code.triangle() + code.square() + code.circle()) % 2 == 1
                }),
            ],
        ),
        // 19
        Verifier::from_description_and_options(
            "the sum of △ and □ compared to 6",
            &[
                VerifierOption::from_description_and_closure("△ + □ < 6", |code| {
                    code.triangle() + code.square() < 6
                }),
                VerifierOption::from_description_and_closure("△ + □ = 6", |code| {
                    code.triangle() + code.square() == 6
                }),
                VerifierOption::from_description_and_closure("△ + □ > 6", |code| {
                    code.triangle() + code.square() > 6
                }),
            ],
        ),
        // 20
        Verifier::from_description_and_options(
            "if a number repeats itself in the code",
            &[
                VerifierOption::from_description_and_closure("a triple number", |code| {
                    code.repeating_numbers() == 2
                }),
                VerifierOption::from_description_and_closure("a double number", |code| {
                    code.repeating_numbers() == 1
                }),
                VerifierOption::from_description_and_closure("no repetition", |code| {
                    code.repeating_numbers() == 0
                }),
            ],
        ),
        // 21
        Verifier::from_description_and_options(
            "if there is a number present exactly twice",
            &[
                VerifierOption::from_description_and_closure("no pairs", |code| {
                    code.repeating_numbers() != 1
                }),
                VerifierOption::from_description_and_closure("a pair", |code| {
                    code.repeating_numbers() == 1
                }),
            ],
        ),
        // 22
        Verifier::from_description_and_options(
            "if the 3 numbers in the code are in ascending order, descending order, or no order",
            &[
                VerifierOption::from_description_and_closure("ascending order", |code| {
                    code.ascending_or_descending() == Order::Ascending
                }),
                VerifierOption::from_description_and_closure("descending order", |code| {
                    code.ascending_or_descending() == Order::Descending
                }),
                VerifierOption::from_description_and_closure("no order", |code| {
                    code.ascending_or_descending() == Order::NoOrder
                }),
            ],
        ),
        // 23
        Verifier::from_description_and_options(
            "the sum of all numbers compared to 6",
            &[
                VerifierOption::from_description_and_closure("△ + □ + ○ < 6", |code| {
                    code.triangle() + code.square() + code.circle() < 6
                }),
                VerifierOption::from_description_and_closure("△ + □ + ○ = 6", |code| {
                    code.triangle() + code.square() + code.circle() == 6
                }),
                VerifierOption::from_description_and_closure("△ + □ + ○ > 6", |code| {
                    code.triangle() + code.square() + code.circle() > 6
                }),
            ],
        ),
        // 24
        Verifier::from_description_and_options(
            "if there is a sequence of ascending numbers",
            &[
                VerifierOption::from_description_and_closure(
                    "3 numbers in ascending order",
                    |code| code.sequence_ascending() == 3,
                ),
                VerifierOption::from_description_and_closure(
                    "2 numbers in ascending order",
                    |code| code.sequence_ascending() == 2,
                ),
                VerifierOption::from_description_and_closure(
                    "no numbers in ascending order",
                    |code| code.sequence_ascending() == 0,
                ),
            ],
        ),
        // 25
        Verifier::from_description_and_options(
            "if there is a sequence of ascending or descending numbers",
            &[
                VerifierOption::from_description_and_closure(
                    "no sequence of numbers in ascending or descending order",
                    |code| code.sequence_ascending_or_descending() == 0,
                ),
                VerifierOption::from_description_and_closure(
                    "2 numbers in ascending or descending order",
                    |code| code.sequence_ascending_or_descending() == 2,
                ),
                VerifierOption::from_description_and_closure(
                    "3 numbers in ascending or descending order",
                    |code| code.sequence_ascending_or_descending() == 3,
                ),
            ],
        ),
        // 26
        Verifier::from_description_and_options(
            "that a specific colour is less than 3",
            &[
                VerifierOption::from_description_and_closure("△ < 3", |code| code.triangle() < 3),
                VerifierOption::from_description_and_closure("□ < 3", |code| code.square() < 3),
                VerifierOption::from_description_and_closure("○ < 3", |code| code.circle() < 3),
            ],
        ),
        // 27
        Verifier::from_description_and_options(
            "that a specific colour is less than 4",
            &[
                VerifierOption::from_description_and_closure("△ < 4", |code| code.triangle() < 4),
                VerifierOption::from_description_and_closure("□ < 4", |code| code.square() < 4),
                VerifierOption::from_description_and_closure("○ < 4", |code| code.circle() < 4),
            ],
        ),
        // 28
        Verifier::from_description_and_options(
            "that a specific colour is equal to 1",
            &[
                VerifierOption::from_description_and_closure("△ = 1", |code| {
                    code.triangle() == 1
                }),
                VerifierOption::from_description_and_closure("□ = 1", |code| code.square() == 1),
                VerifierOption::from_description_and_closure("○ = 1", |code| code.circle() == 1),
            ],
        ),
        // 29
        Verifier::from_description_and_options(
            "that a specific colour is equal to 3",
            &[
                VerifierOption::from_description_and_closure("△ = 3", |code| {
                    code.triangle() == 3
                }),
                VerifierOption::from_description_and_closure("□ = 3", |code| code.square() == 3),
                VerifierOption::from_description_and_closure("○ = 3", |code| code.circle() == 3),
            ],
        ),
        // 30
        Verifier::from_description_and_options(
            "that a specific colour is equal to 4",
            &[
                VerifierOption::from_description_and_closure("△ = 4", |code| {
                    code.triangle() == 4
                }),
                VerifierOption::from_description_and_closure("□ = 4", |code| code.square() == 4),
                VerifierOption::from_description_and_closure("○ = 4", |code| code.circle() == 4),
            ],
        ),
        // 31
        Verifier::from_description_and_options(
            "that a specific colour is greater than 1",
            &[
                VerifierOption::from_description_and_closure("△ > 1", |code| code.triangle() > 1),
                VerifierOption::from_description_and_closure("□ > 1", |code| code.square() > 1),
                VerifierOption::from_description_and_closure("○ > 1", |code| code.circle() > 1),
            ],
        ),
        // 32
        Verifier::from_description_and_options(
            "that a specific colour is greater than 3",
            &[
                VerifierOption::from_description_and_closure("△ > 3", |code| code.triangle() > 3),
                VerifierOption::from_description_and_closure("□ > 3", |code| code.square() > 3),
                VerifierOption::from_description_and_closure("○ > 3", |code| code.circle() > 3),
            ],
        ),
        // 33
        Verifier::from_description_and_options(
            "that a specific colour is even or odd",
            &[
                VerifierOption::from_description_and_closure("△ is even", |code| {
                    code.triangle() % 2 == 0
                }),
                VerifierOption::from_description_and_closure("△ is odd", |code| {
                    code.triangle() % 2 == 1
                }),
                VerifierOption::from_description_and_closure("□ is even", |code| {
                    code.square() % 2 == 0
                }),
                VerifierOption::from_description_and_closure("□ is odd", |code| {
                    code.square() % 2 == 1
                }),
                VerifierOption::from_description_and_closure("○ is even", |code| {
                    code.circle() % 2 == 0
                }),
                VerifierOption::from_description_and_closure("○ is odd", |code| {
                    code.circle() % 2 == 1
                }),
            ],
        ),
        // 34
        Verifier::from_description_and_options(
            "which colour has the smallest number (or is tied for the smallest number)",
            &[
                VerifierOption::from_description_and_closure("△ <= □, ○", |code| {
                    code.triangle() <= code.square() && code.triangle() <= code.circle()
                }),
                VerifierOption::from_description_and_closure("□ <= △, ○", |code| {
                    code.square() <= code.triangle() && code.square() <= code.circle()
                }),
                VerifierOption::from_description_and_closure("○ <= □, △", |code| {
                    code.circle() <= code.square() && code.circle() <= code.triangle()
                }),
            ],
        ),
        // 35
        Verifier::from_description_and_options(
            "which colour has the largest number (or is tied for the largest number)",
            &[
                VerifierOption::from_description_and_closure("△ >= □, ○", |code| {
                    code.triangle() >= code.square() && code.triangle() >= code.circle()
                }),
                VerifierOption::from_description_and_closure("□ >= △, ○", |code| {
                    code.square() >= code.triangle() && code.square() >= code.circle()
                }),
                VerifierOption::from_description_and_closure("○ >= □, △", |code| {
                    code.circle() >= code.square() && code.circle() >= code.triangle()
                }),
            ],
        ),
        // 36
        Verifier::from_description_and_options(
            "the sum of all the numbers is a multiple of 3 or 4 or 5",
            &[
                VerifierOption::from_description_and_closure("△ + □ + ○ = 3x", |code| {
                    code.sum() % 3 == 0
                }),
                VerifierOption::from_description_and_closure("△ + □ + ○ = 4x", |code| {
                    code.sum() % 4 == 0
                }),
                VerifierOption::from_description_and_closure("△ + □ + ○ = 5x", |code| {
                    code.sum() % 5 == 0
                }),
            ],
        ),
        // 37
        Verifier::from_description_and_options(
            "the sum of 2 specific colours is equal to 4",
            &[
                VerifierOption::from_description_and_closure("△ + □ = 4", |code| {
                    code.triangle() + code.square() == 4
                }),
                VerifierOption::from_description_and_closure("△ + ○ = 4", |code| {
                    code.triangle() + code.circle() == 4
                }),
                VerifierOption::from_description_and_closure("□ + ○ = 4", |code| {
                    code.square() + code.circle() == 4
                }),
            ],
        ),
        // 38
        Verifier::from_description_and_options(
            "the sum of 2 specific colours is equal to 6",
            &[
                VerifierOption::from_description_and_closure("△ + □ = 6", |code| {
                    code.triangle() + code.square() == 6
                }),
                VerifierOption::from_description_and_closure("△ + ○ = 6", |code| {
                    code.triangle() + code.circle() == 6
                }),
                VerifierOption::from_description_and_closure("□ + ○ = 6", |code| {
                    code.square() + code.circle() == 6
                }),
            ],
        ),
        // 39
        Verifier::from_description_and_options(
            "the number of one specific colour compared to 1",
            &[
                VerifierOption::from_description_and_closure("△ = 1", |code| {
                    code.triangle() == 1
                }),
                VerifierOption::from_description_and_closure("△ > 1", |code| code.triangle() > 1),
                VerifierOption::from_description_and_closure("□ = 1", |code| code.square() == 1),
                VerifierOption::from_description_and_closure("□ > 1", |code| code.square() > 1),
                VerifierOption::from_description_and_closure("○ = 1", |code| code.circle() == 1),
                VerifierOption::from_description_and_closure("○ > 1", |code| code.circle() > 1),
            ],
        ),
        // 40
        Verifier::from_description_and_options(
            "the number of one specific colour compared to 3",
            &[
                VerifierOption::from_description_and_closure("△ < 3", |code| code.triangle() < 3),
                VerifierOption::from_description_and_closure("△ = 3", |code| {
                    code.triangle() == 3
                }),
                VerifierOption::from_description_and_closure("△ > 3", |code| code.triangle() > 3),
                VerifierOption::from_description_and_closure("□ < 3", |code| code.square() < 3),
                VerifierOption::from_description_and_closure("□ = 3", |code| code.square() == 3),
                VerifierOption::from_description_and_closure("□ > 3", |code| code.square() > 3),
                VerifierOption::from_description_and_closure("○ < 3", |code| code.circle() < 3),
                VerifierOption::from_description_and_closure("○ = 3", |code| code.circle() == 3),
                VerifierOption::from_description_and_closure("○ > 3", |code| code.circle() > 3),
            ],
        ),
        // 41
        Verifier::from_description_and_options(
            "the number of one specific colour compared to 4",
            &[
                VerifierOption::from_description_and_closure("△ < 4", |code| code.triangle() < 4),
                VerifierOption::from_description_and_closure("△ = 4", |code| {
                    code.triangle() == 4
                }),
                VerifierOption::from_description_and_closure("△ > 4", |code| code.triangle() > 4),
                VerifierOption::from_description_and_closure("□ < 4", |code| code.square() < 4),
                VerifierOption::from_description_and_closure("□ = 4", |code| code.square() == 4),
                VerifierOption::from_description_and_closure("□ > 4", |code| code.square() > 4),
                VerifierOption::from_description_and_closure("○ < 4", |code| code.circle() < 4),
                VerifierOption::from_description_and_closure("○ = 4", |code| code.circle() == 4),
                VerifierOption::from_description_and_closure("○ > 4", |code| code.circle() > 4),
            ],
        ),
        // 42
        Verifier::from_description_and_options(
            "which colour is the smallest or the largest",
            &[
                VerifierOption::from_description_and_closure("△ < ○, □", |code| {
                    code.triangle() < code.circle() && code.triangle() < code.square()
                }),
                VerifierOption::from_description_and_closure("△ > ○, □", |code| {
                    code.triangle() > code.circle() && code.triangle() > code.square()
                }),
                VerifierOption::from_description_and_closure("□ < △, ○", |code| {
                    code.square() < code.triangle() && code.square() < code.circle()
                }),
                VerifierOption::from_description_and_closure("□ > △, ○", |code| {
                    code.square() > code.triangle() && code.square() > code.circle()
                }),
                VerifierOption::from_description_and_closure("○ < □, △", |code| {
                    code.circle() < code.square() && code.circle() < code.triangle()
                }),
                VerifierOption::from_description_and_closure("○ > □, △", |code| {
                    code.circle() > code.square() && code.circle() > code.triangle()
                }),
            ],
        ),
        // 43
        Verifier::from_description_and_options(
            "the △ number compared to the number of another specific colour",
            &[
                VerifierOption::from_description_and_closure("△ < □", |code| {
                    code.triangle() < code.square()
                }),
                VerifierOption::from_description_and_closure("△ < ○", |code| {
                    code.triangle() < code.circle()
                }),
                VerifierOption::from_description_and_closure("△ = □", |code| {
                    code.triangle() == code.square()
                }),
                VerifierOption::from_description_and_closure("△ = ○", |code| {
                    code.triangle() == code.circle()
                }),
                VerifierOption::from_description_and_closure("△ > □", |code| {
                    code.triangle() > code.square()
                }),
                VerifierOption::from_description_and_closure("△ > ○", |code| {
                    code.triangle() > code.circle()
                }),
            ],
        ),
        // 44
        Verifier::from_description_and_options(
            "the □ number compared to the number of another specific colour",
            &[
                VerifierOption::from_description_and_closure("□ < △", |code| {
                    code.square() < code.triangle()
                }),
                VerifierOption::from_description_and_closure("□ < ○", |code| {
                    code.square() < code.circle()
                }),
                VerifierOption::from_description_and_closure("□ = △", |code| {
                    code.square() == code.triangle()
                }),
                VerifierOption::from_description_and_closure("□ = ○", |code| {
                    code.square() == code.circle()
                }),
                VerifierOption::from_description_and_closure("□ > △", |code| {
                    code.square() > code.triangle()
                }),
                VerifierOption::from_description_and_closure("□ > ○", |code| {
                    code.square() > code.circle()
                }),
            ],
        ),
        // 45
        Verifier::from_description_and_options(
            "how many 1s OR how many 3s there are in the code",
            &[
                VerifierOption::from_description_and_closure("zero 1s", |code| {
                    code.count_digit(1) == 0
                }),
                VerifierOption::from_description_and_closure("one 1s", |code| {
                    code.count_digit(1) == 1
                }),
                VerifierOption::from_description_and_closure("two 1s", |code| {
                    code.count_digit(1) == 2
                }),
                VerifierOption::from_description_and_closure("zero 3s", |code| {
                    code.count_digit(3) == 0
                }),
                VerifierOption::from_description_and_closure("one 3s", |code| {
                    code.count_digit(3) == 1
                }),
                VerifierOption::from_description_and_closure("two 3s", |code| {
                    code.count_digit(3) == 2
                }),
            ],
        ),
        // 46
        Verifier::from_description_and_options(
            "how many 3s OR how many 4s there are in the code",
            &[
                VerifierOption::from_description_and_closure("zero 3s", |code| {
                    code.count_digit(3) == 0
                }),
                VerifierOption::from_description_and_closure("one 3", |code| {
                    code.count_digit(3) == 1
                }),
                VerifierOption::from_description_and_closure("two 3s", |code| {
                    code.count_digit(3) == 2
                }),
                VerifierOption::from_description_and_closure("zero 4s", |code| {
                    code.count_digit(4) == 0
                }),
                VerifierOption::from_description_and_closure("one 4", |code| {
                    code.count_digit(4) == 1
                }),
                VerifierOption::from_description_and_closure("two 4s", |code| {
                    code.count_digit(4) == 2
                }),
            ],
        ),
        // 47
        Verifier::from_description_and_options(
            "how many 1s OR how many 4s there are in the code",
            &[
                VerifierOption::from_description_and_closure("zero 1s", |code| {
                    code.count_digit(1) == 0
                }),
                VerifierOption::from_description_and_closure("one 1", |code| {
                    code.count_digit(1) == 1
                }),
                VerifierOption::from_description_and_closure("two 1s", |code| {
                    code.count_digit(1) == 2
                }),
                VerifierOption::from_description_and_closure("zero 4s", |code| {
                    code.count_digit(4) == 0
                }),
                VerifierOption::from_description_and_closure("one 4", |code| {
                    code.count_digit(4) == 1
                }),
                VerifierOption::from_description_and_closure("two 4s", |code| {
                    code.count_digit(4) == 2
                }),
            ],
        ),
        // 48
        Verifier::from_description_and_options(
            "one specific colour compared to another specific colour",
            &[
                VerifierOption::from_description_and_closure("△ < □", |code| {
                    code.triangle() < code.square()
                }),
                VerifierOption::from_description_and_closure("△ = □", |code| {
                    code.triangle() == code.square()
                }),
                VerifierOption::from_description_and_closure("△ > □", |code| {
                    code.triangle() > code.square()
                }),
                VerifierOption::from_description_and_closure("△ < ○", |code| {
                    code.triangle() < code.circle()
                }),
                VerifierOption::from_description_and_closure("△ = ○", |code| {
                    code.triangle() == code.circle()
                }),
                VerifierOption::from_description_and_closure("△ > ○", |code| {
                    code.triangle() > code.circle()
                }),
                VerifierOption::from_description_and_closure("□ < ○", |code| {
                    code.square() < code.circle()
                }),
                VerifierOption::from_description_and_closure("□ = ○", |code| {
                    code.square() == code.circle()
                }),
                VerifierOption::from_description_and_closure("□ > ○", |code| {
                    code.square() > code.circle()
                }),
            ],
        ),
    ];
    verifiers[number - 1].clone()
}

const MAX_VERIFIER_OPTIONS: usize = 9;

#[derive(Copy, Clone)]
pub struct VerifierOption {
    pub(crate) description: &'static str,
    code_set: CodeSet,
}

impl VerifierOption {
    pub fn code_set(&self) -> CodeSet {
        self.code_set
    }

    pub fn from_description_and_closure(
        description: &'static str,
        checker: fn(Code) -> bool,
    ) -> VerifierOption {
        VerifierOption {
            description,
            code_set: CodeSet::from_closure(checker),
        }
    }
}

pub(crate) trait Intersection {
    type To;
    fn intersect(self) -> Self::To;
}

impl<T: Iterator<Item = CodeSet>> Intersection for T {
    type To = CodeSet;
    fn intersect(self) -> Self::To {
        self.fold(CodeSet::all(), |still_possible_codes, new_code_set| {
            still_possible_codes.intersected_with(new_code_set)
        })
    }
}

#[derive(Clone)]
pub struct Verifier {
    description: &'static str,
    options: ArrayVec<VerifierOption, MAX_VERIFIER_OPTIONS>,
}

impl Debug for Verifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{}", self.description)?;
        for option in self.options.iter() {
            writeln!(f, "- {}", option.description)?;
        }
        Ok(())
    }
}

impl Verifier {
    pub fn description(&self) -> &'static str {
        self.description
    }

    pub fn number_of_options(&self) -> usize {
        self.options.len()
    }

    pub fn from_description_and_options(
        description: &'static str,
        options: &[VerifierOption],
    ) -> Self {
        Verifier {
            description,
            options: options.iter().copied().collect(),
        }
    }

    pub fn option(&self, choice: u8) -> &VerifierOption {
        &self.options[choice as usize]
    }

    pub fn options(&self) -> impl Iterator<Item = &VerifierOption> + '_ {
        self.options.iter()
    }
}
