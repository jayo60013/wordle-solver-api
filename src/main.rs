mod entropy;
mod filters;
mod models;

use actix_cors::Cors;
use actix_web::middleware::{Compress, Logger};
use entropy::calculate_entropy_for_words;
use filters::filter_words_by_guesses;
use models::Guess;
use serde_json::to_writer;
use std::{
    fs::File,
    io::{self, BufRead, BufReader, Cursor},
    sync::OnceLock,
};

use actix_web::{http, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use std::env;

use crate::models::{PossibleWords, Word};

const ALLOWED_GUESSES_FILENAME: &str = "wordle-nyt-allowed-guesses.txt";
const ANSWERS_FILENAME: &str = "wordle-nyt-answers.txt";
static WORD_LIST: OnceLock<Vec<Word>> = OnceLock::new();

#[post("/possible-words")]
async fn possible_words(guesses: web::Json<Vec<Guess>>) -> impl Responder {
    match WORD_LIST.get() {
        Some(words) => {
            let filtered_words = filter_words_by_guesses(words, &guesses.0);
            let filtered_words_with_entropy = calculate_entropy_for_words(filtered_words);
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
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=info,wordle_solver=info");
    env_logger::init();

    let words = get_all_words_from_file().unwrap();
    if WORD_LIST.set(words).is_err() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to initialise WORD_LIST",
        ));
    };

    info!("Starting HTTP Server on 5307");
    HttpServer::new(|| {
        let cors = Cors::default()
            .allowed_origin("http://localhost:5173")
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::CONTENT_TYPE])
            .max_age(3600);

        App::new()
            .wrap(Logger::default())
            .wrap(cors)
            .wrap(Compress::default())
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
            .filter_map(|line_result| {
                line_result.ok().map(|word| Word {
                    word,
                    entropy: 0.0,
                    is_answer,
                })
            })
            .collect();
        Ok(words)
    }

    let mut words = read_words_from_file(ANSWERS_FILENAME, true)?;
    let allowed_guesses = read_words_from_file(ALLOWED_GUESSES_FILENAME, false)?;

    words.extend(allowed_guesses);
    Ok(words)
}
