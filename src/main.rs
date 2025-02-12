mod filters;
mod models;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use std::io::BufRead;
use std::sync::OnceLock;
use std::{fs::File, io};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use crate::filters::filter_by_letter_contraints;
use crate::models::{LetterConstraints, PossibleWords};

const FILENAME: &str = "word_list.txt";
static WORD_LIST: OnceLock<Vec<String>> = OnceLock::new();

#[get("/all-words")]
async fn all_words() -> impl Responder {
    let word_list = WORD_LIST.get().expect("Global word list not set");
    HttpResponse::Ok().json(PossibleWords {
        word_list: word_list.clone(),
        number_of_words: word_list.len(),
    })
}

#[post("/possible-words")]
async fn possible_words(letter_constraints: web::Json<LetterConstraints>) -> impl Responder {
    let word_list = WORD_LIST.get().expect("Global word list not set");

    let possible_word_list: Vec<String> =
        filter_by_letter_contraints(word_list, letter_constraints.0);
    let number_of_words = possible_word_list.len();
    HttpResponse::Ok().json(PossibleWords {
        word_list: possible_word_list,
        number_of_words,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    set_word_list(FILENAME)?;
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .wrap(Cors::permissive())
            .service(all_words)
            .service(possible_words)
    })
    .bind(("localhost", 5307))?
    .run()
    .await
}

fn set_word_list(filename: &str) -> io::Result<()> {
    let words = read_words(filename)?;
    WORD_LIST
        .set(words)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Global word list is already set"))
}

fn read_words(filename: &str) -> io::Result<Vec<String>> {
    let f = File::open(filename)?;
    let reader = io::BufReader::new(f);

    let lines = reader.lines().collect::<Result<Vec<String>, io::Error>>()?;

    Ok(lines)
}
