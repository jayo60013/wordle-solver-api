mod entropy;
mod errors;
mod filters;
mod models;
mod rate_limit;
mod state;

use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use core::f32;
use entropy::calculate_entropy_for_words;
use errors::ApiError;
use filters::filter_words_by_guesses;
use models::GuessBody;
use rate_limit::IpRateLimiter;
use state::AppState;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
    net::{IpAddr, Ipv4Addr},
};

use actix_web::{post, web, App, HttpResponse, HttpServer, ResponseError};
use log::info;
use std::env;

use crate::models::{PossibleWords, Word};

const ALLOWED_GUESSES_FILENAME: &str = "wordle-nyt-allowed-guesses.txt";
const ANSWERS_FILENAME: &str = "wordle-nyt-answers.txt";

#[post("/possible-words")]
async fn possible_words(
    state: web::Data<AppState>,
    guesses: web::Json<GuessBody>,
    req: actix_web::HttpRequest,
) -> Result<HttpResponse, ApiError> {
    let client_ip = req
        .peer_addr()
        .map_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED), |addr| addr.ip());

    if !state.rate_limiter.check(client_ip) {
        return Err(ApiError::rate_limited(
            "Rate limit exceeded. Maximum 1 request per second allowed.",
            req.path(),
        ));
    }

    if guesses.0 .0.is_empty() {
        return Ok(HttpResponse::Ok().json(&state.empty_guess_cache));
    }

    let filtered_words = filter_words_by_guesses(&state.words, &guesses.0 .0);

    if filtered_words.is_empty() {
        let response = PossibleWords {
            word_list: filtered_words,
            number_of_words: 0,
            total_number_of_words: state.words.len(),
            lowest_entropy: 0.0,
            highest_entropy: 0.0,
        };
        return Ok(HttpResponse::Ok().json(response));
    }

    let filtered_words_with_entropy = calculate_entropy_for_words(&filtered_words);

    let response = PossibleWords {
        word_list: filtered_words_with_entropy.clone(),
        number_of_words: filtered_words_with_entropy.len(),
        total_number_of_words: state.words.len(),
        lowest_entropy: filtered_words_with_entropy
            .iter()
            .map(|w| w.entropy)
            .fold(f32::INFINITY, f32::min),
        highest_entropy: filtered_words_with_entropy
            .iter()
            .map(|w| w.entropy)
            .fold(f32::NEG_INFINITY, f32::max),
    };
    Ok(HttpResponse::Ok().json(response))
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

    // One request per IP per second
    let app_state = web::Data::new(AppState::new(
        words,
        all_words_response,
        IpRateLimiter::new(1, 1.0),
    ));

    info!("Starting HTTP Server on 5307");
    HttpServer::new(move || {
        let cors = Cors::default()
            .allowed_origin("https://wordlesolver.umbra.mom")
            .allowed_methods(vec!["GET", "POST", "OPTIONS"])
            .allow_any_header()
            .max_age(3600);

        let json_cfg = web::JsonConfig::default().error_handler(|err, _req| {
            let api_error = ApiError::bad_request(err.to_string(), _req.path());
            actix_web::error::InternalError::from_response(err, api_error.error_response()).into()
        });

        App::new()
            .app_data(app_state.clone())
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
