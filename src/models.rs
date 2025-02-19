use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LetterConstraints {
    pub grey_letters: Vec<char>,
    pub yellow_letters: Vec<(char, usize)>,
    pub green_letters: Vec<(char, usize)>,
}

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
