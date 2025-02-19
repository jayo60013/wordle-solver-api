use std::collections::HashSet;

use crate::models::{LetterConstraints, Word};

pub fn filter_words_by_letter_contraints(
    words: &[Word],
    letter_constraints: LetterConstraints,
) -> Vec<Word> {
    words
        .iter()
        .filter(|word| filter_by_green_letters(&word.word, &letter_constraints.green_letters))
        .filter(|word| filter_by_yellow_letters(&word.word, &letter_constraints.yellow_letters))
        .filter(|word| filter_by_grey_letters(&word.word, &letter_constraints))
        .cloned()
        .collect()
}

pub fn filter_by_letter_contraints(
    words: &[String],
    letter_constraints: LetterConstraints,
) -> Vec<String> {
    words
        .iter()
        .filter(|word| filter_by_green_letters(word, &letter_constraints.green_letters))
        .filter(|word| filter_by_yellow_letters(word, &letter_constraints.yellow_letters))
        .filter(|word| filter_by_grey_letters(word, &letter_constraints))
        .cloned()
        .collect()
}

fn filter_by_green_letters(word: &str, green_letters: &[(char, usize)]) -> bool {
    // If any word has a green letter not at that position -> word is invalid
    green_letters
        .iter()
        .all(|(letter, position)| word.chars().nth(*position) == Some(*letter))
}

fn filter_by_yellow_letters(word: &str, yellow_letters: &[(char, usize)]) -> bool {
    // If any yellow letter isn't in the word -> word is invalid
    let all_yellow_letters_in_word = yellow_letters
        .iter()
        .all(|(letter, _)| word.contains(*letter));

    if !all_yellow_letters_in_word {
        return false;
    }

    // If any word has a yellow letter at that position -> word is invalid
    yellow_letters
        .iter()
        .all(|(letter, position)| word.chars().nth(*position) != Some(*letter))
}

//TODO: If there a yellow 'a' and a grey 'a'. This means theres 1 'a'
fn filter_by_grey_letters(word: &str, letter_constraints: &LetterConstraints) -> bool {
    // If grey letter - 'a' exists but also green or letter 'a', then we ignore it
    // as we can safely assume previous filters have sorted it

    let yellow_letters: HashSet<&char> = letter_constraints
        .yellow_letters
        .iter()
        .map(|(l, _)| l)
        .collect();
    let green_letters: HashSet<&char> = letter_constraints
        .green_letters
        .iter()
        .map(|(l, _)| l)
        .collect();

    letter_constraints.grey_letters.iter().all(|grey_letter| {
        // If there is none of the same letter as a yellow or a green, simple
        if !(yellow_letters.contains(grey_letter) || green_letters.contains(grey_letter)) {
            return !word.contains(*grey_letter);
        }

        if green_letters.contains(grey_letter) {
            let green_positions: Vec<_> = letter_constraints
                .green_letters
                .iter()
                .filter(|(green_letter, _)| green_letter == grey_letter)
                .map(|(_, position)| position)
                .collect();

            let has_invalid_grey = word
                .chars()
                .enumerate()
                .any(|(i, ch)| ch == *grey_letter && !green_positions.contains(&&i));

            if has_invalid_grey {
                return false;
            }
        }
        true
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_words_by_letter_constraints() {
        // Given
        let letter_constraints = LetterConstraints {
            grey_letters: vec!['u', 'n', 'd', 'l', 'e', 'r', 'm'],
            yellow_letters: vec![('s', 0)],
            green_letters: vec![('o', 1), ('s', 2), ('s', 3), ('y', 4)],
        };
        let input_words = vec![
            Word {
                word: "bossy".to_string(),
                entropy: 0.2,
            },
            Word {
                word: "tossy".to_string(),
                entropy: 0.3,
            },
            Word {
                word: "mossy".to_string(),
                entropy: 0.4,
            },
        ];
        let expected_words = vec![
            Word {
                word: "bossy".to_string(),
                entropy: 0.2,
            },
            Word {
                word: "tossy".to_string(),
                entropy: 0.3,
            },
        ];

        // When
        let actual_words = filter_words_by_letter_contraints(&input_words, letter_constraints);

        // Then
        assert_eq!(actual_words, expected_words);
    }

    #[test]
    fn test_filter_by_letter_constraints() {
        // Given
        let letter_constraints = LetterConstraints {
            grey_letters: vec!['i', 'n', 'd', 'e', 'r', 's', 'f', 't'],
            yellow_letters: vec![('o', 0), ('a', 2), ('l', 3), ('a', 0), ('l', 1), ('o', 2)],
            green_letters: vec![],
        };
        let input_words = vec![
            "loyal".to_string(),
            "vocal".to_string(),
            "mossy".to_string(),
        ];
        let expected_words = vec!["loyal".to_string(), "vocal".to_string()];

        // When
        let actual_words = filter_by_letter_contraints(&input_words, letter_constraints);

        // Then
        assert_eq!(actual_words, expected_words);
    }

    #[test]
    fn test_filter_by_green_letters_does_not_contain_letter() {
        // Given
        let word: String = "share".to_string();
        let letters = vec![('t', 0)];

        // When
        let actual = filter_by_green_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }

    #[test]
    fn test_filter_by_green_letters_contains_letter_at_wrong_position() {
        // Given
        let word: String = "share".to_string();
        let letters = vec![('r', 0)];

        // When
        let actual = filter_by_green_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }

    #[test]
    fn test_filter_by_green_letters_contains_letter_at_correct_position() {
        // Given
        let word: String = "share".to_string();
        let letters = vec![('r', 3)];

        // When
        let actual = filter_by_green_letters(&word, &letters);

        // Then
        assert_eq!(actual, true);
    }

    #[test]
    fn test_filter_by_yellow_letters_does_not_contain_letter() {
        // Given
        let word: String = "apple".to_string();
        let letters = vec![('z', 0), ('p', 4)];

        // When
        let actual = filter_by_yellow_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }

    #[test]
    fn test_filter_by_yellow_letters_contains_letter_at_position() {
        // Given
        let word: String = "apple".to_string();
        let letters = vec![('a', 0)];

        // When
        let actual = filter_by_yellow_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }

    #[test]
    fn test_filter_by_yellow_letters_valid() {
        // Given
        let word: String = "share".to_string();
        let letters = vec![('r', 0), ('s', 3)];

        // When
        let actual = filter_by_yellow_letters(&word, &letters);

        // Then
        assert_eq!(actual, true);
    }

    #[test]
    fn test_filter_by_grey_letters_false() {
        // Given
        let word: String = "apple".to_string();
        let letters = LetterConstraints {
            grey_letters: vec!['a'],
            yellow_letters: vec![],
            green_letters: vec![],
        };

        // When
        let actual = filter_by_grey_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }

    #[test]
    fn test_filter_by_grey_letters_true() {
        // Given
        let word: String = "reign".to_string();
        let letters = LetterConstraints {
            grey_letters: vec!['a', 'b', 'c', 'd'],
            yellow_letters: vec![],
            green_letters: vec![],
        };

        // When
        let actual = filter_by_grey_letters(&word, &letters);

        // Then
        assert_eq!(actual, true);
    }

    #[test]
    fn test_filter_by_grey_letters_when_yellow_exists() {
        // Given
        let word: String = "overt".to_string();
        let letters = LetterConstraints {
            grey_letters: vec!['b', 'l', 'o', 'd'],
            yellow_letters: vec![('o', 2)],
            green_letters: vec![],
        };

        // When
        let actual = filter_by_grey_letters(&word, &letters);

        // Then
        assert_eq!(actual, true);
    }

    #[test]
    fn test_filter_by_grey_letters_when_green_exists() {
        // Given
        let word: String = "aloof".to_string();
        let letters = LetterConstraints {
            grey_letters: vec!['b', 'o', 'd', 'g', 'v', 'e'],
            yellow_letters: vec![],
            green_letters: vec![('l', 1), ('o', 2)],
        };

        // When
        let actual = filter_by_grey_letters(&word, &letters);

        // Then
        assert_eq!(actual, false);
    }
}
