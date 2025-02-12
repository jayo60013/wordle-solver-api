use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct LetterConstraints {
    pub grey_letters: Vec<char>,
    pub yellow_letters: Vec<(char, usize)>,
    pub green_letters: Vec<(char, usize)>,
}

#[derive(Serialize)]
pub struct PossibleWords {
    pub word_list: Vec<String>,
    pub number_of_words: usize,
}
