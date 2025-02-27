use crate::{
    filters::filter_words_by_guesses,
    models::{Color, Guess, Word},
};
use itertools::Itertools;
use rayon::prelude::*;
use std::iter::repeat;

pub fn calculate_entropy_for_words(words: Vec<Word>) -> Vec<Word> {
    words
        .par_iter()
        .enumerate()
        .map(|(i, word)| {
            let entropy = calculate_entropy_for_word(word, &words);
            println!("({}/{}) {}: {}", i, words.len(), word.word, entropy);
            Word {
                word: word.word.clone(),
                entropy,
                is_answer: word.is_answer,
            }
        })
        .collect::<Vec<Word>>()
        .into_iter()
        .sorted_by(|a, b| b.entropy.total_cmp(&a.entropy))
        .collect()
}

fn calculate_entropy_for_word(word: &Word, words: &[Word]) -> f32 {
    let colors = [Color::Grey, Color::Yellow, Color::Green];
    repeat(colors)
        .take(5)
        .multi_cartesian_product()
        .map(|perm| {
            let guess: Vec<Guess> = perm
                .par_iter()
                .enumerate()
                .map(|(position, color)| Guess {
                    //TODO: create new struct and remove turn
                    turn: 0,
                    letter: word.word.chars().nth(position).unwrap(),
                    position,
                    color: *color,
                })
                .collect();
            calculate_entropy_from_one_guess(words, guess)
        })
        .sum()
}

fn calculate_entropy_from_one_guess(words: &[Word], guess: Vec<Guess>) -> f32 {
    let total_number_of_words = words.len();
    if total_number_of_words == 0 {
        return 0.0;
    }

    let number_of_possible_words = filter_words_by_guesses(words, &guess).len();
    if number_of_possible_words == 0 {
        return 0.0;
    }

    let probability = number_of_possible_words as f32 / total_number_of_words as f32;
    probability * probability.recip().log2()
}
