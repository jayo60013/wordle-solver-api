mod filters;
mod models;

use actix_cors::Cors;
use actix_web::middleware::Logger;
use std::io::BufRead;
use std::sync::OnceLock;
use std::time::SystemTime;
use std::{fs::File, io};

use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder};

use crate::filters::{filter_by_letter_contraints, filter_words_by_letter_contraints};
use crate::models::{LetterConstraints, PossibleWords, Word};

const FILENAME: &str = "word_list.txt";
static WORD_LIST: OnceLock<Vec<Word>> = OnceLock::new();

#[get("/all-words")]
async fn all_words() -> impl Responder {
    let word_list = WORD_LIST.get().expect("Global word list not set");
    HttpResponse::Ok().json(PossibleWords {
        word_list: word_list.clone(),
        number_of_words: word_list.len(),
        total_number_of_words: word_list.len(),
    })
}

#[post("/possible-words")]
async fn possible_words(letter_constraints: web::Json<LetterConstraints>) -> impl Responder {
    let word_list = WORD_LIST.get().expect("Global word list not set");

    let possible_word_list: Vec<Word> =
        filter_words_by_letter_contraints(word_list, letter_constraints.0);
    let number_of_words = possible_word_list.len();
    HttpResponse::Ok().json(PossibleWords {
        word_list: possible_word_list,
        number_of_words,
        total_number_of_words: word_list.len(),
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let end = SystemTime::now();
    set_word_list(FILENAME)?;
    let duration = end.elapsed();
    println!("set_word_list took: {:.2?}", duration);
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
    let total = words.len() as f32;

    let mut word_structs: Vec<Word> = words
        .iter()
        .map(|word| {
            let grey_letters: Vec<char> = word.chars().collect();
            let letter_constraints = LetterConstraints {
                grey_letters,
                yellow_letters: vec![],
                green_letters: vec![],
            };
            let filtered_words = filter_by_letter_contraints(&words, letter_constraints);
            let amount_of_filtered_words = filtered_words.len();

            Word {
                word: word.clone(),
                entropy: amount_of_filtered_words as f32 / total,
            }
        })
        .collect();

    word_structs.sort_by(|a, b| a.entropy.partial_cmp(&b.entropy).unwrap());

    WORD_LIST
        .set(word_structs)
        .map_err(|_| io::Error::new(io::ErrorKind::Other, "Global word list is already set"))
}

fn read_words(filename: &str) -> io::Result<Vec<String>> {
    let f = File::open(filename)?;
    let reader = io::BufReader::new(f);

    let lines = reader.lines().collect::<Result<Vec<String>, io::Error>>()?;

    Ok(lines)
}
