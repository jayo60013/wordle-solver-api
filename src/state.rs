use crate::models::{PossibleWords, Word};
use crate::rate_limit::IpRateLimiter;

pub struct AppState {
    pub words: Vec<Word>,
    pub empty_guess_cache: PossibleWords,
    pub rate_limiter: IpRateLimiter,
}

impl AppState {
    pub fn new(
        words: Vec<Word>,
        empty_guess_cache: PossibleWords,
        rate_limiter: IpRateLimiter,
    ) -> Self {
        Self {
            words,
            empty_guess_cache,
            rate_limiter,
        }
    }
}

