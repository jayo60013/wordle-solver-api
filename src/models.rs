use serde::{Deserialize, Serialize};
use std::convert::TryFrom;

#[derive(Serialize)]
pub struct PossibleWords {
    pub word_list: Vec<Word>,
    pub number_of_words: usize,
    pub total_number_of_words: usize,
    pub lowest_entropy: f32,
    pub highest_entropy: f32,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize)]
pub struct Word {
    pub word: String,
    pub entropy: f32,
    pub is_answer: bool,
    #[serde(skip)]
    pub bytes: [u8; 5],
}

impl Word {
    pub fn new(word: String, is_answer: bool) -> Self {
        let b = word.as_bytes();
        let bytes = [b[0], b[1], b[2], b[3], b[4]];
        Word { word, entropy: 0.0, is_answer, bytes }
    }
}

#[derive(Deserialize, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Debug)]
pub enum Color {
    Grey,
    Yellow,
    Green,
}

#[derive(Deserialize, Clone)]
pub struct Guess {
    pub turn: usize,
    pub letter: char,
    pub position: usize,
    pub color: Color,
}

#[derive(Deserialize)]
#[serde(try_from = "Vec<Guess>")]
pub struct GuessBody(pub Vec<Guess>);

impl TryFrom<Vec<Guess>> for GuessBody {
    type Error = String;

    fn try_from(guesses: Vec<Guess>) -> Result<Self, Self::Error> {
        if guesses.is_empty() || !guesses.len().is_multiple_of(5) {
            return Err("All guesses must have 5 letters.".to_string());
        }
        Ok(GuessBody(guesses))
    }
}

#[cfg(test)]
mod tests {
    use super::{Color, Guess, GuessBody};

    fn construct_guess(turn: usize, letter: char, position: usize, color: Color) -> Guess {
        Guess {
            turn,
            letter,
            position,
            color,
        }
    }

    #[test]
    fn guess_body_try_from_ok() {
        // Given
        let guesses = vec![
            construct_guess(0, 'c', 0, Color::Green),
            construct_guess(0, 'r', 1, Color::Grey),
            construct_guess(0, 'a', 2, Color::Yellow),
            construct_guess(0, 'n', 3, Color::Grey),
            construct_guess(0, 'e', 4, Color::Grey),
        ];

        // When
        let result = GuessBody::try_from(guesses.clone());

        // Then
        assert!(result.is_ok());
    }

    #[test]
    fn guess_body_try_from_rejects_empty() {
        // Given
        let guesses = vec![];

        // When
        let result = GuessBody::try_from(guesses);

        assert_eq!(result.err(), Some("All guesses must have 5 letters.".to_string()));
    }

    #[test]
    fn guess_body_try_from_rejects_non_multiple_of_five_lengths() {
        // Given
        let guesses = vec![construct_guess(0, 'c', 0, Color::Green); 4];

        // When
        let result = GuessBody::try_from(guesses);

        // Then
        assert_eq!(result.err(), Some("All guesses must have 5 letters.".to_string()));
    }

}