use crate::models::{Color, Guess, Word};

fn filter_words_by_guesses(words: Vec<Word>, guesses: &[Guess]) -> Vec<Word> {
    words
        .into_iter()
        .filter(|word| filter_word_by_guesses(word.clone(), guesses))
        .collect()
}

fn filter_word_by_guesses(word: Word, guesses: &[Guess]) -> bool {
    guesses.iter().all(|guess| match guess.color {
        Color::GREEN => return word.word.chars().nth(guess.position).unwrap() == guess.letter,
        Color::YELLOW => {
            if !word.word.contains(guess.letter) {
                return false;
            }

            word.word.chars().nth(guess.position).unwrap() != guess.letter
        }
        Color::GREY => {
            let same_turn: Vec<Guess> = guesses
                .iter()
                .filter(|g| g.turn == guess.turn && g.letter == guess.letter)
                .cloned()
                .collect();

            let greens: Vec<Guess> = same_turn
                .iter()
                .filter(|g| g.color == Color::GREEN)
                .cloned()
                .collect();
            let yellows: Vec<Guess> = same_turn
                .iter()
                .filter(|g| g.color == Color::GREEN)
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
                color: Color::GREY,
            },
            Guess {
                turn: 0,
                letter: 'l',
                position: 1,
                color: Color::GREY,
            },
            Guess {
                turn: 0,
                letter: 'o',
                position: 2,
                color: Color::YELLOW,
            },
            Guess {
                turn: 0,
                letter: 'o',
                position: 3,
                color: Color::GREEN,
            },
            Guess {
                turn: 0,
                letter: 'd',
                position: 4,
                color: Color::GREY,
            },
        ];
        let input_words = vec![
            Word {
                word: "aloof".to_string(),
                entropy: 0.0,
            },
            Word {
                word: "order".to_string(),
                entropy: 0.0,
            },
            Word {
                word: "achoo".to_string(),
                entropy: 0.0,
            },
            Word {
                word: "cocoa".to_string(),
                entropy: 0.0,
            },
        ];
        let expected_words = vec![
            Word {
                word: "achoo".to_string(),
                entropy: 0.0,
            },
            Word {
                word: "cocoa".to_string(),
                entropy: 0.0,
            },
        ];

        // When
        let actual_words = filter_words_by_guesses(input_words, &guesses);

        // Then
        assert_eq!(actual_words, expected_words);
    }
}
