use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct PossibleWords {
    pub word_list: Vec<Word>,
    pub number_of_words: usize,
    pub total_number_of_words: usize,
}

#[derive(Debug, PartialEq, PartialOrd, Clone, Serialize)]
pub struct Word {
    pub word: String,
    pub entropy: f32,
}

#[derive(Deserialize, PartialEq, Eq, Clone, Copy, PartialOrd, Ord)]
pub enum Color {
    GREY,
    YELLOW,
    GREEN,
}

#[derive(Deserialize, Clone)]
pub struct Guess {
    pub turn: usize,
    pub letter: char,
    pub position: usize,
    pub color: Color,
}
