use crate::rate_limit::IpRateLimiter;
use crate::{
    entropy::FeedbackTable,
    models::{PossibleWords, Word},
};

pub struct AppState {
    pub words: Vec<Word>,
    pub answers: Vec<Word>,
    pub legal_guesses: Vec<Word>,
    pub feedback_table: FeedbackTable,
    pub empty_guess_cache: PossibleWords,
    pub rate_limiter: IpRateLimiter,
}

impl AppState {
    pub fn new(
        words: Vec<Word>,
        answers: Vec<Word>,
        legal_guesses: Vec<Word>,
        feedback_table: FeedbackTable,
        empty_guess_cache: PossibleWords,
        rate_limiter: IpRateLimiter,
    ) -> Self {
        Self {
            words,
            answers,
            legal_guesses,
            feedback_table,
            empty_guess_cache,
            rate_limiter,
        }
    }
}
