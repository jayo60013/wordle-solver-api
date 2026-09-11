use std::collections::HashSet;

use rayon::prelude::*;
use serde::Serialize;

use crate::{
    entropy::{compute_pattern, FeedbackTable},
    models::{Color, Guess, Word},
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GameAnalysis {
    pub answer: String,
    pub turns: usize,
    pub skill: Score,
    pub luck: Score,
    pub turn_analysis: Vec<TurnAnalysis>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Score {
    pub score_percent: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TurnAnalysis {
    pub turn: usize,
    pub possible_answer_count_before: usize,
    pub possible_answer_count_after: usize,
    pub skill_percent: f64,
    pub luck_percent: f64,
    pub best_guesses: Vec<RankedGuess>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RankedGuess {
    pub word: String,
    pub entropy_percent: f64,
    pub is_answer: bool,
}

struct TurnScore {
    entropy: f64,
    information: f64,
    variance: f64,
    luck_percent: f64,
    possible_answer_count_after: usize,
}

pub fn analyse_game(
    guesses: &[Guess],
    answers: &[Word],
    legal_guesses: &[Word],
    feedback_table: &FeedbackTable,
) -> Result<GameAnalysis, String> {
    let rows = validate_game(guesses, answers, legal_guesses)?;
    let answer = rows
        .last()
        .expect("validated game has a final row")
        .0
        .clone();
    let answer_index = answers
        .iter()
        .position(|word| word.word == answer)
        .expect("validated answer exists in answer list");

    let mut prior_guesses = Vec::new();
    let mut turn_analysis = Vec::with_capacity(rows.len());
    let mut total_entropy = 0.0;
    let mut total_best_entropy = 0.0;
    let mut total_luck_delta = 0.0;
    let mut total_luck_variance = 0.0;
    let legal_word_indices = legal_guesses
        .iter()
        .enumerate()
        .map(|(index, word)| (word.word.as_str(), index))
        .collect::<std::collections::HashMap<_, _>>();
    let mut candidate_indices = (0..answers.len()).collect::<Vec<_>>();

    for (turn, (word, row)) in rows.iter().enumerate() {
        let eligible_guesses = legal_guesses
            .iter()
            .enumerate()
            .filter(|(_, guess)| satisfies_hard_mode(guess, &prior_guesses))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();

        let mut ranked = eligible_guesses
            .par_iter()
            .map(|guess_index| {
                (
                    *guess_index,
                    entropy_for_guess(*guess_index, &candidate_indices, feedback_table),
                )
            })
            .collect::<Vec<_>>();
        let best_entropy = ranked
            .iter()
            .map(|(_, entropy)| *entropy)
            .max_by(|left, right| left.total_cmp(right))
            .unwrap_or(0.0);
        let top_guess_count = ranked.len().min(5);
        if top_guess_count > 0 {
            ranked.select_nth_unstable_by(
                top_guess_count - 1,
                |(left_index, left_entropy), (right_index, right_entropy)| {
                    right_entropy.total_cmp(left_entropy).then_with(|| {
                        legal_guesses[*left_index]
                            .word
                            .cmp(&legal_guesses[*right_index].word)
                    })
                },
            );
            ranked.truncate(top_guess_count);
            ranked.sort_by(|(left_index, left_entropy), (right_index, right_entropy)| {
                right_entropy.total_cmp(left_entropy).then_with(|| {
                    legal_guesses[*left_index]
                        .word
                        .cmp(&legal_guesses[*right_index].word)
                })
            });
        }

        let selected_index = legal_word_indices[word.as_str()];
        let score = score_outcome(
            selected_index,
            answer_index,
            &candidate_indices,
            feedback_table,
        );
        let maximum_entropy = (candidate_indices.len() as f64).log2();

        total_entropy += score.entropy;
        total_best_entropy += best_entropy;
        total_luck_delta += score.information - score.entropy;
        total_luck_variance += score.variance;

        turn_analysis.push(TurnAnalysis {
            turn,
            possible_answer_count_before: candidate_indices.len(),
            possible_answer_count_after: score.possible_answer_count_after,
            skill_percent: percent_of(score.entropy, best_entropy, 100.0),
            luck_percent: score.luck_percent,
            best_guesses: ranked
                .iter()
                .take(5)
                .map(|(guess_index, entropy)| RankedGuess {
                    word: legal_guesses[*guess_index].word.clone(),
                    entropy_percent: percent_of(*entropy, maximum_entropy, 0.0),
                    is_answer: legal_guesses[*guess_index].is_answer,
                })
                .collect(),
        });

        prior_guesses.extend(row.iter().cloned());
        let selected_pattern = encode_pattern(row);
        candidate_indices.retain(|answer_index| {
            feedback_table.pattern(selected_index, *answer_index) == selected_pattern
        });
    }

    Ok(GameAnalysis {
        answer,
        turns: rows.len(),
        skill: Score {
            score_percent: percent_of(total_entropy, total_best_entropy, 100.0),
        },
        luck: Score {
            score_percent: overall_luck_percent(total_luck_delta, total_luck_variance),
        },
        turn_analysis,
    })
}

fn validate_game(
    guesses: &[Guess],
    answers: &[Word],
    legal_guesses: &[Word],
) -> Result<Vec<(String, Vec<Guess>)>, String> {
    if guesses.is_empty() || guesses.len() > 30 || !guesses.len().is_multiple_of(5) {
        return Err("A game must contain between 1 and 6 complete rows.".to_string());
    }

    let turn_count = guesses.len() / 5;
    let legal_words = legal_guesses
        .iter()
        .map(|word| word.word.as_str())
        .collect::<HashSet<_>>();
    let answer_words = answers
        .iter()
        .map(|word| word.word.as_str())
        .collect::<HashSet<_>>();
    let mut rows = Vec::with_capacity(turn_count);

    for turn in 0..turn_count {
        let row = guesses
            .iter()
            .filter(|guess| guess.turn == turn)
            .cloned()
            .collect::<Vec<_>>();
        if row.len() != 5 {
            return Err(
                "Turns must be contiguous and each must contain exactly 5 letters.".to_string(),
            );
        }

        let mut letters = [' '; 5];
        for guess in &row {
            if guess.position >= 5 || !guess.letter.is_ascii_lowercase() {
                return Err(
                    "Letters must be lowercase ASCII characters in positions 0 through 4."
                        .to_string(),
                );
            }
            if letters[guess.position] != ' ' {
                return Err("Each position may appear only once per turn.".to_string());
            }
            letters[guess.position] = guess.letter;
        }
        if letters.contains(&' ') {
            return Err("Each turn must contain positions 0 through 4.".to_string());
        }

        let word = letters.iter().collect::<String>();
        if !legal_words.contains(word.as_str()) {
            return Err(format!("'{word}' is not a legal NYT Wordle guess."));
        }
        rows.push((word, row));
    }

    let final_word = &rows.last().expect("game is not empty").0;
    if !rows
        .last()
        .expect("game is not empty")
        .1
        .iter()
        .all(|guess| guess.color == Color::Green)
    {
        return Err("The final row must be a winning guess with five green letters.".to_string());
    }
    if !answer_words.contains(final_word.as_str()) {
        return Err("The winning word must be an NYT Wordle answer.".to_string());
    }

    let answer = answers
        .iter()
        .find(|word| word.word == *final_word)
        .expect("validated answer exists");
    for (word, row) in &rows {
        let guess = legal_guesses
            .iter()
            .find(|candidate| candidate.word == *word)
            .expect("validated guess exists");
        if encode_pattern(row) != compute_pattern(guess.bytes, answer.bytes) {
            return Err(format!(
                "Feedback for '{word}' does not match the winning answer."
            ));
        }
    }
    if rows[..rows.len() - 1]
        .iter()
        .any(|(_, row)| row.iter().all(|guess| guess.color == Color::Green))
    {
        return Err("A game cannot contain guesses after a winning row.".to_string());
    }

    let mut prior_guesses = Vec::new();
    for (word, row) in &rows {
        let guess = legal_guesses
            .iter()
            .find(|candidate| candidate.word == *word)
            .expect("validated guess exists");
        if !satisfies_hard_mode(guess, &prior_guesses) {
            return Err(format!(
                "'{word}' does not satisfy hard-mode clues from previous turns."
            ));
        }
        prior_guesses.extend(row.iter().cloned());
    }

    Ok(rows)
}

fn satisfies_hard_mode(word: &Word, prior_guesses: &[Guess]) -> bool {
    let mut minimum_counts = [0usize; 26];
    for turn in 0..=prior_guesses
        .iter()
        .map(|guess| guess.turn)
        .max()
        .unwrap_or(0)
    {
        let mut turn_counts = [0usize; 26];
        for guess in prior_guesses.iter().filter(|guess| guess.turn == turn) {
            if guess.color == Color::Green && word.bytes[guess.position] != guess.letter as u8 {
                return false;
            }
            if guess.color == Color::Yellow && word.bytes[guess.position] == guess.letter as u8 {
                return false;
            }
            if matches!(guess.color, Color::Green | Color::Yellow) {
                let index = (guess.letter as u8 - b'a') as usize;
                turn_counts[index] += 1;
            }
        }
        for (index, count) in turn_counts.into_iter().enumerate() {
            minimum_counts[index] = minimum_counts[index].max(count);
        }
    }

    let mut word_counts = [0usize; 26];
    for letter in word.bytes {
        word_counts[(letter - b'a') as usize] += 1;
    }
    word_counts
        .iter()
        .zip(minimum_counts)
        .all(|(actual, minimum)| *actual >= minimum)
}

fn entropy_for_guess(
    guess_index: usize,
    candidate_indices: &[usize],
    feedback_table: &FeedbackTable,
) -> f64 {
    let buckets = pattern_buckets(guess_index, candidate_indices, feedback_table);
    let total = candidate_indices.len() as f64;
    buckets
        .into_iter()
        .filter(|count| *count > 0)
        .map(|count| {
            let probability = count as f64 / total;
            -probability * probability.log2()
        })
        .sum()
}

fn score_outcome(
    guess_index: usize,
    answer_index: usize,
    candidate_indices: &[usize],
    feedback_table: &FeedbackTable,
) -> TurnScore {
    let buckets = pattern_buckets(guess_index, candidate_indices, feedback_table);
    let total = candidate_indices.len() as f64;
    let observed_count = buckets[feedback_table.pattern(guess_index, answer_index) as usize];
    let observed_probability = observed_count as f64 / total;
    let information = -observed_probability.log2();
    let entropy = buckets
        .into_iter()
        .filter(|count| *count > 0)
        .map(|count| {
            let probability = count as f64 / total;
            -probability * probability.log2()
        })
        .sum::<f64>();
    let variance = buckets
        .into_iter()
        .filter(|count| *count > 0)
        .map(|count| {
            let probability = count as f64 / total;
            let bucket_information = -probability.log2();
            probability * (bucket_information - entropy).powi(2)
        })
        .sum();
    let luck_percent = 100.0
        * buckets
            .into_iter()
            .filter(|count| *count >= observed_count)
            .map(|count| count as f64 / total)
            .sum::<f64>();
    TurnScore {
        entropy,
        information,
        variance,
        luck_percent,
        possible_answer_count_after: observed_count as usize,
    }
}

fn pattern_buckets(
    guess_index: usize,
    candidate_indices: &[usize],
    feedback_table: &FeedbackTable,
) -> [u16; 243] {
    let mut buckets = [0u16; 243];
    for answer_index in candidate_indices {
        buckets[feedback_table.pattern(guess_index, *answer_index) as usize] += 1;
    }
    buckets
}

fn percent_of(value: f64, total: f64, no_opportunity_value: f64) -> f64 {
    if total == 0.0 {
        no_opportunity_value
    } else {
        (100.0 * value / total).clamp(-100.0, 100.0)
    }
}

fn overall_luck_percent(delta: f64, variance: f64) -> f64 {
    if variance == 0.0 {
        return 50.0;
    }
    normal_cdf(delta / variance.sqrt()) * 100.0
}

fn normal_cdf(value: f64) -> f64 {
    let t = 1.0 / (1.0 + 0.231_641_9 * value.abs());
    let density = (-value * value / 2.0).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let probability = 1.0
        - density
            * (0.319_381_530 * t - 0.356_563_782 * t.powi(2) + 1.781_477_937 * t.powi(3)
                - 1.821_255_978 * t.powi(4)
                + 1.330_274_429 * t.powi(5));
    if value >= 0.0 {
        probability
    } else {
        1.0 - probability
    }
}

fn encode_pattern(row: &[Guess]) -> u8 {
    row.iter().fold(0u8, |pattern, guess| {
        pattern
            + match guess.color {
                Color::Grey => 0,
                Color::Yellow => 3u8.pow(guess.position as u32),
                Color::Green => 2 * 3u8.pow(guess.position as u32),
            }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn word(value: &str, is_answer: bool) -> Word {
        Word::new(value.to_string(), is_answer)
    }

    fn row(turn: usize, value: &str, colours: [Color; 5]) -> Vec<Guess> {
        value
            .chars()
            .enumerate()
            .map(|(position, letter)| Guess {
                turn,
                letter,
                position,
                color: colours[position],
            })
            .collect()
    }

    #[test]
    fn analyses_a_valid_hard_mode_win() {
        let answers = vec![word("cigar", true), word("rebut", true)];
        let legal_guesses = vec![word("cigar", true), word("rebut", true)];
        let feedback_table = FeedbackTable::new(&legal_guesses, &answers);
        let mut guesses = row(
            0,
            "rebut",
            [
                Color::Yellow,
                Color::Grey,
                Color::Grey,
                Color::Grey,
                Color::Grey,
            ],
        );
        guesses.extend(row(1, "cigar", [Color::Green; 5]));

        let result = analyse_game(&guesses, &answers, &legal_guesses, &feedback_table).unwrap();

        assert_eq!(result.answer, "cigar");
        assert_eq!(result.turn_analysis.len(), 2);
        assert!(result.turn_analysis[0].best_guesses.len() <= 5);
    }

    #[test]
    fn rejects_a_guess_that_ignores_a_green_hard_mode_clue() {
        let answers = vec![word("cigar", true), word("cider", true)];
        let legal_guesses = vec![
            word("cigar", true),
            word("cider", true),
            word("rebut", false),
        ];
        let feedback_table = FeedbackTable::new(&legal_guesses, &answers);
        let mut guesses = row(
            0,
            "cider",
            [
                Color::Green,
                Color::Green,
                Color::Grey,
                Color::Grey,
                Color::Green,
            ],
        );
        guesses.extend(row(
            1,
            "rebut",
            [
                Color::Yellow,
                Color::Grey,
                Color::Grey,
                Color::Grey,
                Color::Grey,
            ],
        ));
        guesses.extend(row(2, "cigar", [Color::Green; 5]));

        assert!(matches!(
            analyse_game(&guesses, &answers, &legal_guesses, &feedback_table),
            Err(message) if message.contains("hard-mode")
        ));
    }
}
