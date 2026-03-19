mod entropy;
mod filters;
mod models;
mod rate_limit;

use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use core::f32;
use entropy::calculate_entropy_for_words;
use filters::filter_words_by_guesses;
use models::GuessBody;
use rate_limit::IpRateLimiter;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    sync::OnceLock,
};

use actix_web::{post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use std::env;

use crate::models::{PossibleWords, Word};

const ALLOWED_GUESSES_FILENAME: &str = "wordle-nyt-allowed-guesses.txt";
const ANSWERS_FILENAME: &str = "wordle-nyt-answers.txt";
static WORD_LIST: OnceLock<Vec<Word>> = OnceLock::new();
static RATE_LIMITER: OnceLock<IpRateLimiter> = OnceLock::new();
static EMPTY_GUESS_CACHE: OnceLock<PossibleWords> = OnceLock::new();

#[post("/possible-words")]
async fn possible_words(
    guesses: web::Json<GuessBody>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let client_ip = req
        .peer_addr()
        .map_or_else(|| "0.0.0.0".parse().unwrap(), |addr| addr.ip());

    if let Some(limiter) = RATE_LIMITER.get() {
        if !limiter.check(client_ip) {
            return HttpResponse::TooManyRequests()
                .content_type("application/json")
                .body(
                    r#"{"error":"Rate limit exceeded. Maximum 1 requests per second allowed."}"#,
                );
        }
    }

    if guesses.0 .0.is_empty() {
        if let Some(cached) = EMPTY_GUESS_CACHE.get() {
            return HttpResponse::Ok().json(cached);
        }
    }

    match WORD_LIST.get() {
        Some(words) => {
            let filtered_words = filter_words_by_guesses(words, &guesses.0 .0);

            if filtered_words.is_empty() {
                let response = PossibleWords {
                    word_list: filtered_words,
                    number_of_words: 0,
                    total_number_of_words: words.len(),
                    lowest_entropy: 0.0,
                    highest_entropy: 0.0,
                };
                return HttpResponse::Ok().json(response);
            }

            let filtered_words_with_entropy = calculate_entropy_for_words(&filtered_words);
            let number_of_words = filtered_words_with_entropy.len();
            let total_number_of_words = words.len();
            let lowest_entropy: f32 = filtered_words_with_entropy
                .iter()
                .min_by(|a, b| a.entropy.partial_cmp(&b.entropy).unwrap())
                .map(|w| w.entropy)
                .unwrap();
            let highest_entropy: f32 = filtered_words_with_entropy
                .iter()
                .max_by(|a, b| a.entropy.partial_cmp(&b.entropy).unwrap())
                .map(|w| w.entropy)
                .unwrap();

            let response = PossibleWords {
                word_list: filtered_words_with_entropy,
                number_of_words,
                total_number_of_words,
                lowest_entropy,
                highest_entropy,
            };
            HttpResponse::Ok().json(response)
        }
        None => HttpResponse::InternalServerError()
            .content_type("application/json")
            .body(r#"{"error":"Word list not initialised"}"#),
    }
}

#[actix_web::main]
async fn main() -> io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=info,wordle_solver=info");
    env_logger::init();

    let words = get_all_words_from_file()?;
    let all_words_entropy = calculate_entropy_for_words(&words);

    let all_words_response = PossibleWords {
        word_list: all_words_entropy.clone(),
        number_of_words: words.len(),
        total_number_of_words: words.len(),
        lowest_entropy: all_words_entropy
            .iter()
            .map(|w| w.entropy)
            .fold(f32::INFINITY, f32::min),
        highest_entropy: all_words_entropy
            .iter()
            .map(|w| w.entropy)
            .fold(f32::NEG_INFINITY, f32::max),
    };

    WORD_LIST
        .set(words)
        .map_err(|_| io::Error::other("Failed to initialise WORD_LIST"))?;
    EMPTY_GUESS_CACHE
        .set(all_words_response)
        .map_err(|_| io::Error::other("Failed to initialise EMPTY_GUESS_CACHE"))?;

    // One request per IP per second
    let limiter = IpRateLimiter::new(1, 1.0);
    RATE_LIMITER
        .set(limiter)
        .map_err(|_| io::Error::other("Failed to initialise RATE_LIMITER"))?;

    info!("Starting HTTP Server on 5307");
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("https://wordlesolver.umbra.mom")
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allow_any_header()
            .max_age(3600);

        let json_cfg = web::JsonConfig::default().error_handler(|err, _req| {
            let body = format!("{{\"error\":\"{err}\"}}");
            actix_web::error::InternalError::from_response(
                err,
                HttpResponse::BadRequest()
                    .content_type("application/json")
                    .body(body),
            )
            .into()
        });

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .wrap(Compress::default())
            .app_data(json_cfg)
            .service(possible_words)
    })
    .bind(("0.0.0.0", 5307))?
    .run()
    .await
}

fn get_all_words_from_file() -> io::Result<Vec<Word>> {
    fn read_words_from_file(filename: &str, is_answer: bool) -> io::Result<Vec<Word>> {
        let file = File::open(filename)?;
        let r = BufReader::new(file);
        let words: Vec<Word> = r
            .lines()
            .filter_map(|line_result| line_result.ok().map(|word| Word::new(word, is_answer)))
            .collect();
        Ok(words)
    }

    let mut words = read_words_from_file(ANSWERS_FILENAME, true)?;
    let allowed_guesses = read_words_from_file(ALLOWED_GUESSES_FILENAME, false)?;

    words.extend(allowed_guesses);
    Ok(words)
}
