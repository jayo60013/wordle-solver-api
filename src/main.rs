mod filters;
mod models;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use models::Guess;
use std::sync::OnceLock;

use actix_web::{http, post, web, App, HttpResponse, HttpServer, Responder};
use log::info;
use std::env;

use crate::models::{PossibleWords, Word};

const FILENAME: &str = "wordle-nyt-answers.txt";
static WORD_LIST: OnceLock<Vec<Word>> = OnceLock::new();

#[post("/possible-words")]
async fn possible_words(guesses: web::Json<Vec<Guess>>) -> impl Responder {
    //let word_list = WORD_LIST.get().expect("Global word list not set");
    //
    //let possible_word_list: Vec<Word> =
    //    filter_words_by_letter_contraints(word_list, letter_constraints.0);
    //let number_of_words = possible_word_list.len();
    //HttpResponse::Ok().json(PossibleWords {
    //    word_list: possible_word_list,
    //    number_of_words,
    //    total_number_of_words: word_list.len(),
    //})
    HttpResponse::Ok().body("Hello")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "actix_web=info,wordle_solver=info");
    env_logger::init();

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
            .service(possible_words)
    })
    .bind(("0.0.0.0", 5307))?
    .run()
    .await
}
