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
    let rows = validate_game(guesses, answers)?;
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

        let selected_entropy = legal_word_indices
            .get(word.as_str())
            .map(|selected_index| {
                entropy_for_guess(*selected_index, &candidate_indices, feedback_table)
            })
            .unwrap_or_else(|| entropy_for_custom_guess(word, answers, &candidate_indices));
        let score = score_outcome(word, answer_index, answers, &candidate_indices, feedback_table);
        let maximum_entropy = (candidate_indices.len() as f64).log2();

        total_entropy += selected_entropy;
        total_best_entropy += best_entropy;
        total_luck_delta += score.information - selected_entropy;
        total_luck_variance += score.variance;

        turn_analysis.push(TurnAnalysis {
            turn,
            possible_answer_count_before: candidate_indices.len(),
            possible_answer_count_after: score.possible_answer_count_after,
            skill_percent: percent_of(selected_entropy, best_entropy, 100.0),
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
        let selected_word = Word::new(word.clone(), false);
        candidate_indices.retain(|answer_index| {
            compute_pattern(selected_word.bytes, answers[*answer_index].bytes) == selected_pattern
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
) -> Result<Vec<(String, Vec<Guess>)>, String> {
    if guesses.is_empty() || guesses.len() > 30 || !guesses.len().is_multiple_of(5) {
        return Err("A game must contain between 1 and 6 complete rows.".to_string());
    }

    let turn_count = guesses.len() / 5;
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
        let word = letters.iter().collect::<String>();
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
        let guess = Word::new(word.clone(), answer.word == *word);
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
        let guess = Word::new(word.clone(), answer.word == *word);
        if !satisfies_hard_mode(&guess, &prior_guesses) {
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
    guess_word: &str,
    answer_index: usize,
    answers: &[Word],
    candidate_indices: &[usize],
    feedback_table: &FeedbackTable,
) -> TurnScore {
    let buckets =
        pattern_buckets_for_guess(guess_word, answers, candidate_indices, Some(feedback_table));
    let total = candidate_indices.len() as f64;
    let observed_pattern = compute_pattern(
        Word::new(guess_word.to_string(), false).bytes,
        answers[answer_index].bytes,
    ) as usize;
    let observed_count = buckets[observed_pattern];
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
        information,
        variance,
        luck_percent,
        possible_answer_count_after: observed_count as usize,
    }
}

fn entropy_for_custom_guess(guess_word: &str, answers: &[Word], candidate_indices: &[usize]) -> f64 {
    pattern_buckets_for_guess(guess_word, answers, candidate_indices, None)
        .into_iter()
        .filter(|count| *count > 0)
        .map(|count| {
            let probability = count as f64 / candidate_indices.len() as f64;
            -probability * probability.log2()
        })
        .sum()
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

fn pattern_buckets_for_guess(
    guess_word: &str,
    answers: &[Word],
    candidate_indices: &[usize],
    feedback_table: Option<&FeedbackTable>,
) -> [u16; 243] {
    if let Some((guess_index, _)) = feedback_table.and_then(|table| {
        answers
            .iter()
            .position(|word| word.word == guess_word)
            .map(|index| (index, table))
    })
    {
        return pattern_buckets(guess_index, candidate_indices, feedback_table.expect("table exists"));
    }

    let guess = Word::new(guess_word.to_string(), false);
    let mut buckets = [0u16; 243];
    for answer_index in candidate_indices {
        buckets[compute_pattern(guess.bytes, answers[*answer_index].bytes) as usize] += 1;
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

    fn feedback_row(turn: usize, value: &str, answer: &str) -> Vec<Guess> {
        let guess = word(value, false);
        let answer = word(answer, true);
        let pattern = compute_pattern(guess.bytes, answer.bytes);
        row(
            turn,
            value,
            std::array::from_fn(|position| match (pattern / 3u8.pow(position as u32)) % 3 {
                0 => Color::Grey,
                1 => Color::Yellow,
                2 => Color::Green,
                _ => unreachable!("a Wordle pattern is base three"),
            }),
        )
    }

    fn analysis_inputs() -> (Vec<Word>, Vec<Word>, FeedbackTable) {
        let answers = vec![word("cigar", true), word("rebut", true)];
        let legal_guesses = vec![
            word("cigar", true),
            word("rebut", true),
            word("adieu", false),
            word("stare", false),
            word("slate", false),
            word("crane", false),
        ];
        let feedback_table = FeedbackTable::new(&legal_guesses, &answers);

        (answers, legal_guesses, feedback_table)
    }

    fn error_message(result: Result<GameAnalysis, String>) -> String {
        match result {
            Ok(_) => panic!("expected game analysis to fail"),
            Err(message) => message,
        }
    }

    #[test]
    fn rejects_invalid_game_lengths() {
        // Given
        let (answers, legal_guesses, feedback_table) = analysis_inputs();
        let empty_game = Vec::new();
        let incomplete_game = vec![
            Guess {
                turn: 0,
                letter: 'c',
                position: 0,
                color: Color::Grey,
            };
            4
        ];
        let too_long_game = vec![
            Guess {
                turn: 0,
                letter: 'c',
                position: 0,
                color: Color::Grey,
            };
            35
        ];

        // When
        let empty_result = analyse_game(&empty_game, &answers, &legal_guesses, &feedback_table);
        let incomplete_result =
            analyse_game(&incomplete_game, &answers, &legal_guesses, &feedback_table);
        let too_long_result =
            analyse_game(&too_long_game, &answers, &legal_guesses, &feedback_table);

        // Then
        assert_eq!(
            error_message(empty_result),
            "A game must contain between 1 and 6 complete rows."
        );
        assert_eq!(
            error_message(incomplete_result),
            "A game must contain between 1 and 6 complete rows."
        );
        assert_eq!(
            error_message(too_long_result),
            "A game must contain between 1 and 6 complete rows."
        );
    }

    #[test]
    fn rejects_non_contiguous_turns() {
        // Given
        let (answers, legal_guesses, feedback_table) = analysis_inputs();
        let mut guesses = feedback_row(0, "rebut", "cigar");
        guesses.extend(feedback_row(2, "cigar", "cigar"));

        // When
        let result = analyse_game(&guesses, &answers, &legal_guesses, &feedback_table);

        // Then
        assert_eq!(
            error_message(result),
            "Turns must be contiguous and each must contain exactly 5 letters."
        );
    }

    #[test]
    fn rejects_invalid_letter_or_position() {
        // Given
        let (answers, legal_guesses, feedback_table) = analysis_inputs();
        let invalid_letter = row(0, "cigar", [Color::Green; 5])
            .into_iter()
            .enumerate()
            .map(|(position, mut guess)| {
                if position == 0 {
                    guess.letter = 'C';
                }
                guess
            })
            .collect::<Vec<_>>();
        let invalid_position = row(0, "cigar", [Color::Green; 5])
            .into_iter()
            .enumerate()
            .map(|(position, mut guess)| {
                if position == 4 {
                    guess.position = 5;
                }
                guess
            })
            .collect::<Vec<_>>();

        // When
        let letter_result =
            analyse_game(&invalid_letter, &answers, &legal_guesses, &feedback_table);
        let position_result =
            analyse_game(&invalid_position, &answers, &legal_guesses, &feedback_table);

        // Then
        assert_eq!(
            error_message(letter_result),
            "Letters must be lowercase ASCII characters in positions 0 through 4."
        );
        assert_eq!(
            error_message(position_result),
            "Letters must be lowercase ASCII characters in positions 0 through 4."
        );
    }

    #[test]
    fn rejects_duplicate_positions() {
        // Given
        let (answers, legal_guesses, feedback_table) = analysis_inputs();
        let guesses = row(0, "cigar", [Color::Green; 5])
            .into_iter()
            .enumerate()
            .map(|(index, mut guess)| {
                if index == 4 {
                    guess.position = 3;
                }
                guess
            })
            .collect::<Vec<_>>();

        // When
        let result = analyse_game(&guesses, &answers, &legal_guesses, &feedback_table);

        // Then
        assert_eq!(
            error_message(result),
            "Each position may appear only once per turn."
        );
    }

    #[test]
    fn allows_non_wordle_guesses_but_rejects_invalid_winning_words() {
        // Given
        let (answers, legal_guesses, feedback_table) = analysis_inputs();
        let mut unconventional_guess = row(0, "xxxxx", [Color::Grey; 5]);
        unconventional_guess.extend(row(1, "cigar", [Color::Green; 5]));
        let non_answer_win = row(0, "adieu", [Color::Green; 5]);

        // When
        let unconventional_result =
            analyse_game(&unconventional_guess, &answers, &legal_guesses, &feedback_table);
        let non_answer_result =
            analyse_game(&non_answer_win, &answers, &legal_guesses, &feedback_table);

        // Then
        assert!(unconventional_result.is_ok());
        assert_eq!(
            error_message(non_answer_result),
            "The winning word must be an NYT Wordle answer."
        );
    }

    #[test]
    fn rejects_non_winning_final_rows_and_incorrect_feedback() {
        // Given
        let (answers, legal_guesses, feedback_table) = analysis_inputs();
        let non_winning_final_row = row(0, "rebut", [Color::Grey; 5]);
        let mut incorrect_feedback = row(0, "rebut", [Color::Grey; 5]);
        incorrect_feedback.extend(row(1, "cigar", [Color::Green; 5]));

        // When
        let final_row_result = analyse_game(
            &non_winning_final_row,
            &answers,
            &legal_guesses,
            &feedback_table,
        );
        let feedback_result = analyse_game(
            &incorrect_feedback,
            &answers,
            &legal_guesses,
            &feedback_table,
        );

        // Then
        assert_eq!(
            error_message(final_row_result),
            "The final row must be a winning guess with five green letters."
        );
        assert_eq!(
            error_message(feedback_result),
            "Feedback for 'rebut' does not match the winning answer."
        );
    }

    #[test]
    fn rejects_games_continued_after_a_win() {
        // Given
        let (answers, legal_guesses, feedback_table) = analysis_inputs();
        let mut guesses = feedback_row(0, "cigar", "cigar");
        guesses.extend(feedback_row(1, "cigar", "cigar"));

        // When
        let result = analyse_game(&guesses, &answers, &legal_guesses, &feedback_table);

        // Then
        assert_eq!(
            error_message(result),
            "A game cannot contain guesses after a winning row."
        );
    }

    #[test]
    fn analyses_hard_mode_win_with_ranked_top_five_and_metrics() {
        // Given
        let (answers, legal_guesses, feedback_table) = analysis_inputs();
        let mut guesses = feedback_row(0, "rebut", "cigar");
        guesses.extend(feedback_row(1, "cigar", "cigar"));

        // When
        let result = analyse_game(&guesses, &answers, &legal_guesses, &feedback_table).unwrap();

        // Then
        assert_eq!(result.answer, "cigar");
        assert_eq!(result.turns, 2);
        assert_eq!(result.turn_analysis[0].possible_answer_count_before, 2);
        assert_eq!(result.turn_analysis[0].possible_answer_count_after, 1);
        assert_eq!(result.turn_analysis[0].best_guesses.len(), 5);
        assert_eq!(
            result.turn_analysis[0]
                .best_guesses
                .iter()
                .map(|guess| guess.word.as_str())
                .collect::<Vec<_>>(),
            vec!["adieu", "cigar", "crane", "rebut", "slate"]
        );
        assert!(result.turn_analysis[0]
            .best_guesses
            .windows(2)
            .all(|pair| pair[0].entropy_percent >= pair[1].entropy_percent));
        assert_eq!(result.turn_analysis[1].best_guesses.len(), 3);
        assert_eq!(result.turn_analysis[1].best_guesses[0].word, "cigar");
        assert_eq!(result.turn_analysis[1].skill_percent, 100.0);
        assert_eq!(result.skill.score_percent, 100.0);
        assert!(result.luck.score_percent.is_finite());
    }

    #[test]
    fn rejects_a_guess_that_ignores_a_green_hard_mode_clue() {
        // Given
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

        // When
        let result = analyse_game(&guesses, &answers, &legal_guesses, &feedback_table);

        // Then
        assert_eq!(
            error_message(result),
            "'rebut' does not satisfy hard-mode clues from previous turns."
        );
    }

    #[test]
    fn hard_mode_requires_yellows_elsewhere_and_known_letter_multiplicity() {
        // Given
        let yellow_r_at_first_position = row(
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
        let two_yellow_a_letters = row(
            1,
            "aaaab",
            [
                Color::Yellow,
                Color::Yellow,
                Color::Grey,
                Color::Grey,
                Color::Grey,
            ],
        );
        let yellow_in_same_position = word("rebut", false);
        let missing_second_a = word("cigar", true);
        let valid_word = word("bbaca", false);

        // When
        let same_position_result =
            satisfies_hard_mode(&yellow_in_same_position, &yellow_r_at_first_position);
        let missing_count_result = satisfies_hard_mode(&missing_second_a, &two_yellow_a_letters);
        let valid_result = satisfies_hard_mode(&valid_word, &two_yellow_a_letters);

        // Then
        assert!(!same_position_result);
        assert!(!missing_count_result);
        assert!(valid_result);
    }

    #[test]
    fn scores_single_candidate_games_without_entropy_or_luck_variance() {
        // Given
        let answers = vec![word("cigar", true)];
        let legal_guesses = vec![word("cigar", true)];
        let feedback_table = FeedbackTable::new(&legal_guesses, &answers);
        let guesses = row(0, "cigar", [Color::Green; 5]);

        // When
        let result = analyse_game(&guesses, &answers, &legal_guesses, &feedback_table).unwrap();

        // Then
        assert_eq!(result.turn_analysis[0].skill_percent, 100.0);
        assert_eq!(result.turn_analysis[0].luck_percent, 100.0);
        assert_eq!(result.skill.score_percent, 100.0);
        assert_eq!(result.luck.score_percent, 50.0);
    }

    #[test]
    fn numerical_helpers_handle_zero_totals_bounds_and_both_normal_tails() {
        // Given
        let positive_value = 1.0;
        let negative_value = -1.0;

        // When
        let no_opportunity = percent_of(2.0, 0.0, 42.0);
        let upper_bound = percent_of(3.0, 1.0, 0.0);
        let lower_bound = percent_of(-3.0, 1.0, 0.0);
        let neutral_luck = overall_luck_percent(10.0, 0.0);
        let positive_tail = normal_cdf(positive_value);
        let negative_tail = normal_cdf(negative_value);
        let positive_luck = overall_luck_percent(positive_value, 1.0);

        // Then
        assert_eq!(no_opportunity, 42.0);
        assert_eq!(upper_bound, 100.0);
        assert_eq!(lower_bound, -100.0);
        assert_eq!(neutral_luck, 50.0);
        assert!((positive_tail - 0.8413).abs() < 0.0001);
        assert!((negative_tail - 0.1587).abs() < 0.0001);
        assert!((positive_luck - positive_tail * 100.0).abs() < f64::EPSILON);
    }
}
