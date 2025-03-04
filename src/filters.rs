use crate::models::{Color, Guess, Word};

pub fn filter_words_by_guesses(words: &[Word], guesses: &[Guess]) -> Vec<Word> {
    words
        .iter()
        .filter(|word| filter_word_by_guesses(word, guesses))
        .cloned()
        .collect()
}

fn filter_word_by_guesses(word: &Word, guesses: &[Guess]) -> bool {
    let word_chars: Vec<char> = word.word.chars().collect();

    guesses.iter().all(|guess| {
        let expected_total = get_expected_total_of_letters(guesses, guess);
        let actual_total = word_chars.iter().filter(|&&c| c == guess.letter).count();

        match guess.color {
            Color::Green => word_chars[guess.position] == guess.letter,
            Color::Yellow => {
                actual_total >= expected_total && word_chars[guess.position] != guess.letter
            }
            Color::Grey => {
                expected_total == actual_total && word_chars[guess.position] != guess.letter
            }
        }
    })
}

fn get_expected_total_of_letters(guesses: &[Guess], guess: &Guess) -> usize {
    guesses
        .iter()
        .filter(|g| g.turn == guess.turn && g.letter == guess.letter)
        .fold(0, |acc, g| match g.color {
            Color::Green | Color::Yellow => acc + 1,
            _ => acc,
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

    #[test]
    fn test_grey_letter_cant_appear_in_place_its_appeared_before() {
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
                position: 1,
                color: Color::Grey,
            },
            Guess {
                turn: 0,
                letter: 'r',
                position: 2,
                color: Color::Grey,
            },
            Guess {
                turn: 0,
                letter: 'i',
                position: 3,
                color: Color::Grey,
            },
            Guess {
                turn: 0,
                letter: 'e',
                position: 4,
                color: Color::Grey,
            },
        ];
        let input_words = vec![
            Word {
                word: "taste".to_string(),
                entropy: 0.0,
                is_answer: true,
            },
            Word {
                word: "asset".to_string(),
                entropy: 0.0,
                is_answer: true,
            },
        ];
        let expected_words = vec![Word {
            word: "asset".to_string(),
            entropy: 0.0,
            is_answer: true,
        }];

        // When
        let actual_words = filter_words_by_guesses(&input_words, &guesses);

        // Then
        assert_eq!(actual_words, expected_words);
    }
}
