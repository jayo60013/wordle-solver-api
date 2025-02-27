use crate::models::{Color, Guess, Word};

pub fn filter_words_by_guesses(words: &[Word], guesses: &[Guess]) -> Vec<Word> {
    words
        .iter()
        .filter(|word| filter_word_by_guesses(word, guesses))
        .cloned()
        .collect()
}

fn filter_word_by_guesses(word: &Word, guesses: &[Guess]) -> bool {
    guesses.iter().all(|guess| match guess.color {
        Color::Green => word.word.chars().nth(guess.position).unwrap() == guess.letter,
        Color::Yellow => {
            let number_of_greens = guesses
                .iter()
                .filter(|g| {
                    g.turn == guess.turn && g.letter == guess.letter && g.color == Color::Green
                })
                .cloned()
                .count();
            let number_of_yellows = guesses
                .iter()
                .filter(|g| {
                    g.turn == guess.turn && g.letter == guess.letter && g.color == Color::Yellow
                })
                .cloned()
                .count();
            let expected_total = number_of_greens + number_of_yellows;
            let actual_total = word.word.chars().filter(|&c| c == guess.letter).count();

            if actual_total < expected_total {
                return false;
            }

            word.word.chars().nth(guess.position).unwrap() != guess.letter
        }
        Color::Grey => {
            let same_turn: Vec<Guess> = guesses
                .iter()
                .filter(|g| g.turn == guess.turn && g.letter == guess.letter)
                .cloned()
                .collect();

            let greens: Vec<Guess> = same_turn
                .iter()
                .filter(|g| g.color == Color::Green)
                .cloned()
                .collect();
            let yellows: Vec<Guess> = same_turn
                .iter()
                .filter(|g| g.color == Color::Yellow)
                .cloned()
                .collect();

            // This is the total number of expected letters in the word
            let expected_total = greens.len() + yellows.len();
            let actual_total = word.word.chars().filter(|&c| c == guess.letter).count();
            expected_total == actual_total
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_words_by_guesses() {
        // Given
        let guesses = [
            Guess {
                turn: 0,
                letter: 'b',
                position: 0,
                color: Color::Grey,
            },
            Guess {
                turn: 0,
                letter: 'l',
                position: 1,
                color: Color::Grey,
            },
            Guess {
                turn: 0,
                letter: 'o',
                position: 2,
                color: Color::Yellow,
            },
            Guess {
                turn: 0,
                letter: 'o',
                position: 3,
                color: Color::Green,
            },
            Guess {
                turn: 0,
                letter: 'd',
                position: 4,
                color: Color::Grey,
            },
        ];
        let input_words = vec![
            Word {
                word: "aloof".to_string(),
                entropy: 0.0,
                is_answer: true,
            },
            Word {
                word: "xenon".to_string(),
                entropy: 0.0,
                is_answer: true,
            },
            Word {
                word: "achoo".to_string(),
                entropy: 0.0,
                is_answer: true,
            },
            Word {
                word: "cocoa".to_string(),
                entropy: 0.0,
                is_answer: true,
            },
        ];
        let expected_words = vec![
            Word {
                word: "achoo".to_string(),
                entropy: 0.0,
                is_answer: true,
            },
            Word {
                word: "cocoa".to_string(),
                entropy: 0.0,
                is_answer: true,
            },
        ];

        // When
        let actual_words = filter_words_by_guesses(&input_words, &guesses);

        // Then
        assert_eq!(actual_words, expected_words);
    }

    #[test]
    fn test_double_yellow_letter_gives_word_with_double_and_more() {
        // Given
        let guesses = [
            Guess {
                turn: 0,
                letter: 'e',
                position: 0,
                color: Color::Yellow,
            },
            Guess {
                turn: 0,
                letter: 'e',
                position: 4,
                color: Color::Yellow,
            },
        ];
        let input_words = vec![
            Word {
                word: "tenet".to_string(),
                entropy: 0.0,
                is_answer: true,
            },
            Word {
                word: "tests".to_string(),
                entropy: 0.0,
                is_answer: true,
            },
        ];
        let expected_words = vec![Word {
            word: "tenet".to_string(),
            entropy: 0.0,
            is_answer: true,
        }];

        // When
        let actual_words = filter_words_by_guesses(&input_words, &guesses);

        // Then
        assert_eq!(actual_words, expected_words);
    }
}
