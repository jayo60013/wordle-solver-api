use std::collections::HashMap;

use crate::models::{Color, Guess, Word};
use rayon::prelude::*;

pub fn filter_words_by_guesses(words: &[Word], guesses: &[Guess]) -> Vec<Word> {
    words
        .par_iter()
        .filter(|word| filter_word_by_guesses(word, guesses))
        .cloned()
        .collect()
}

fn filter_word_by_guesses(word: &Word, guesses: &[Guess]) -> bool {
    let word_chars = word.word.as_bytes();
    let char_counts = word_chars.iter().fold(HashMap::new(), |mut acc, &c| {
        *acc.entry(c).or_insert(0) += 1;
        acc
    });

    guesses.iter().all(|guess| {
        let expected_total = get_expected_total_of_letters(guesses, guess);
        let actual_total = *char_counts.get(&(guess.letter as u8)).unwrap_or(&0);

        match guess.color {
            Color::Green => word_chars[guess.position] == guess.letter as u8,
            Color::Yellow => {
                actual_total >= expected_total && word_chars[guess.position] != guess.letter as u8
            }
            Color::Grey => {
                expected_total == actual_total && word_chars[guess.position] != guess.letter as u8
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
            Color::Grey => acc,
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
            Word::new("aloof".to_string(), true),
            Word::new("xenon".to_string(), true),
            Word::new("achoo".to_string(), true),
            Word::new("cocoa".to_string(), true),
        ];
        let expected_words = vec![
            Word::new("achoo".to_string(), true),
            Word::new("cocoa".to_string(), true),
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
            Word::new("tenet".to_string(), true),
            Word::new("tests".to_string(), true),
        ];
        let expected_words = vec![Word::new("tenet".to_string(), true)];

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
            Word::new("taste".to_string(), true),
            Word::new("asset".to_string(), true),
        ];
        let expected_words = vec![Word::new("asset".to_string(), true)];

        // When
        let actual_words = filter_words_by_guesses(&input_words, &guesses);

        // Then
        assert_eq!(actual_words, expected_words);
    }
}
