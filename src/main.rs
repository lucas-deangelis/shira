use actix_web::{http::StatusCode, web, App, HttpResponse, HttpServer};
use rand::distributions::Alphanumeric;
use rand::{Rng, SeedableRng};
use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::sync::Mutex;

static INDEX: &str = include_str!("index.html");
static PASTE_FOLDER: &str = "pastes/";
static FILENAME_LENGTH: usize = 4;

#[derive(Deserialize)]
struct FormData {
    code: String,
}

struct FileCreator {
    rng: rand::rngs::StdRng,
}

impl FileCreator {
    fn new() -> FileCreator {
        FileCreator {
            rng: rand::prelude::StdRng::from_entropy(),
        }
    }

    fn create_file(&mut self) -> String {
        let filename = loop {
            let name: String = std::iter::repeat(())
                .map(|()| self.rng.sample(Alphanumeric))
                .map(char::from)
                .take(FILENAME_LENGTH)
                .collect();
            if !Path::new(&format!("{}{}", PASTE_FOLDER, name)).exists() {
                break name;
            }
        };

        let complete_path = format!("{}{}", PASTE_FOLDER, filename);
        fs::File::create(complete_path).expect("Couldn't create file");

        filename
    }
}

async fn index() -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(INDEX)
}

async fn file(filename: web::Path<String>) -> HttpResponse {
    match fs::read_to_string(format!("{}{}", PASTE_FOLDER, filename)) {
        Ok(content) => HttpResponse::build(StatusCode::OK)
            .content_type("text/plain; charset=utf-8")
            .body(content),
        Err(_) => HttpResponse::NotFound().body("Paste not found"),
    }
}

async fn receive_form(
    form: web::Form<FormData>,
    file_creator: web::Data<Mutex<FileCreator>>,
) -> HttpResponse {
    let filename = {
        // Creating a scope here so the lock is dropped as early as possible
        let mut borrowed_file_creator = file_creator.lock().unwrap();
        borrowed_file_creator.create_file()
    };

    let complete_path = format!("{}{}", PASTE_FOLDER, filename);
    fs::write(complete_path, form.code.clone()).expect("Unable to write to file");

    HttpResponse::Found()
        .header(actix_web::http::header::LOCATION, filename)
        .finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if !Path::new("pastes/").exists() {
        match fs::create_dir("pastes") {
            Ok(_) => (),
            Err(_) => panic!("No folder /pastes, and couldn't create it"),
        }
    }

    let file_creator = web::Data::new(Mutex::new(FileCreator::new()));

    HttpServer::new(move || {
        App::new()
            .app_data(file_creator.clone())
            .route("/", web::get().to(index))
            .route("/", web::post().to(receive_form))
            .route("/{filename}", web::get().to(file))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
