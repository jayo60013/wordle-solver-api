use crate::models::Word;
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashMap;

pub fn calculate_entropy_for_words(words: &[Word]) -> Vec<Word> {
    let word_bytes: Vec<[u8; 5]> = words.iter().map(|w| w.bytes).collect();

    let answer_count = words.iter().filter(|w| w.is_answer).count().max(1) as f32;

    words
        .par_iter()
        .enumerate()
        .map(|(i, word)| {
            let extra_letters: u8 = word
                .word
                .chars()
                .fold(HashMap::new(), |mut acc, c| {
                    *acc.entry(c).or_insert(0u8) += 1;
                    acc
                })
                .values()
                .map(|&count| count - 1)
                .sum();

            let penalty_multiplier = 0.85_f32.powi(i32::from(extra_letters));

            let answer_bonus = if word.is_answer {
                answer_count.log2() / answer_count
            } else {
                0.0
            };

            let raw_entropy = calculate_entropy_for_word(word_bytes[i], &word_bytes);
            Word {
                word: word.word.clone(),
                entropy: raw_entropy * penalty_multiplier + answer_bonus,
                is_answer: word.is_answer,
                bytes: word.bytes,
            }
        })
        .collect::<Vec<Word>>()
        .into_iter()
        .sorted_by(|a, b| b.entropy.total_cmp(&a.entropy))
        .collect()
}

fn compute_pattern(guess: [u8; 5], candidate: [u8; 5]) -> u8 {
    let mut counts = [0u8; 26];
    for b in candidate {
        counts[(b - b'a') as usize] += 1;
    }

    let mut pattern = [0u8; 5]; // 0 = grey

    for i in 0..5 {
        if guess[i] == candidate[i] {
            pattern[i] = 2;
            counts[(guess[i] - b'a') as usize] -= 1;
        }
    }

    for i in 0..5 {
        if pattern[i] != 2 {
            let idx = (guess[i] - b'a') as usize;
            if counts[idx] > 0 {
                pattern[i] = 1;
                counts[idx] -= 1;
            }
        }
    }

    // Encode as base-3 u8 (fits in a single byte, 0–242).
    let mut encoded = 0u8;
    let mut mult = 1u8;
    for p in pattern {
        encoded += p * mult;
        mult *= 3;
    }
    encoded
}

fn calculate_entropy_for_word(guess: [u8; 5], candidates: &[[u8; 5]]) -> f32 {
    let total = candidates.len() as f32;
    let mut buckets = [0u32; 243];

    for candidate in candidates {
        buckets[compute_pattern(guess, *candidate) as usize] += 1;
    }

    buckets
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f32 / total;
            p * p.recip().log2()
        })
        .sum()
}
