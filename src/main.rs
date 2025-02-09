mod filters;

use serde::Deserialize;
use std::io::BufRead;
use std::sync::OnceLock;
use std::{fs::File, io};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use filters::{filter_by_green_letters, filter_by_grey_letters, filter_by_yellow_letters};

const FILENAME: &str = "word_list.txt";
static WORD_LIST: OnceLock<Vec<String>> = OnceLock::new();

#[derive(Debug, Deserialize)]
struct LetterConstraints {
    grey_letters: Vec<char>,
    yellow_letters: Vec<(char, usize)>,
    green_letters: Vec<(char, usize)>,
}

#[get("/all-words")]
async fn all_words() -> impl Responder {
    let word_list = WORD_LIST.get().expect("Global word list not set");
    HttpResponse::Ok().json(word_list)
}

#[post("/possible-words")]
async fn possible_words(letter_constraints: web::Json<LetterConstraints>) -> impl Responder {
    let word_list = WORD_LIST.get().expect("Global word list not set");

    let possible_word_list: Vec<&String> = word_list
        .iter()
        .filter(|word| filter_by_grey_letters(word, &letter_constraints.grey_letters))
        .filter(|word| filter_by_yellow_letters(word, &letter_constraints.yellow_letters))
        .filter(|word| filter_by_green_letters(word, &letter_constraints.green_letters))
        .collect();
    HttpResponse::Ok().json(possible_word_list)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    set_word_list(FILENAME)?;
    HttpServer::new(|| App::new().service(all_words).service(possible_words))
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
