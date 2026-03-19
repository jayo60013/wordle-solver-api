mod entropy;
mod filters;
mod models;
mod rate_limit;

use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use entropy::calculate_entropy_for_words;
use filters::filter_words_by_guesses;
use models::GuessBody;
use rate_limit::IpRateLimiter;
use serde_json::to_writer;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Cursor},
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

    match WORD_LIST.get() {
        Some(words) => {
            let filtered_words = filter_words_by_guesses(words, &guesses.0 .0);
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

            let mut cursor = Cursor::new(Vec::new());
            if to_writer(&mut cursor, &response).is_ok() {
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(cursor.into_inner())
            } else {
                HttpResponse::InternalServerError().finish()
            }
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
    if WORD_LIST.set(words).is_err() {
        return Err(std::io::Error::other("Failed to initialise WORD_LIST"));
    }

    let limiter = IpRateLimiter::new(1, 1.0);
    if RATE_LIMITER.set(limiter).is_err() {
        return Err(io::Error::other("Failed to initialise RATE_LIMITER"));
    }

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
