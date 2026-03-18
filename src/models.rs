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
        if guesses.is_empty() || guesses.len().is_multiple_of(5) {
            return Err("All guesses must have 5 letters.".to_string());
        }
        Ok(GuessBody(guesses))
    }
}