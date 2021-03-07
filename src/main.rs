use actix_web::{App, HttpResponse, HttpServer, http::StatusCode, web};
use serde::Deserialize;
use rand::Rng;
use rand::distributions::Alphanumeric;
use std::path::Path;
use std::fs;

static INDEX: &'static str = include_str!("index.html");
static PASTE_FOLDER: &'static str = "pastes/";
static FILENAME_LENGTH: usize = 4;

#[derive(Deserialize)]
struct FormData {
    code: String,
}

fn create_filename() -> String {
    let mut rng = rand::thread_rng();

    let filename = loop {
        let name: String = std::iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .map(char::from)
            .take(FILENAME_LENGTH)
            .collect();
        if !Path::new(&format!("{}{}", PASTE_FOLDER, name)).exists() {
            break name;
        }
    };

    filename
}

async fn index() -> HttpResponse {
    HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(INDEX)
}

async fn file(filename: web::Path<String>) -> HttpResponse {
    match fs::read_to_string(format!("{}{}", PASTE_FOLDER, filename)) {
        Ok(content) => HttpResponse::Ok().body(content),
        Err(_) => HttpResponse::NotFound().body("Paste not found"),
    }
}

async fn receive_form(form: web::Form<FormData>) -> HttpResponse {
    let filename = create_filename();
    let complete_path = format!("{}{}", PASTE_FOLDER, filename);
    fs::File::create(complete_path.clone()).expect("Couldn't create file");
    fs::write(complete_path.clone(), form.code.clone()).expect("Unable to write to file");

    HttpResponse::Found().header(actix_web::http::header::LOCATION, filename).finish()
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    if !Path::new("pastes/").exists() {
        match fs::create_dir("pastes") {
            Ok(_) => (),
            Err(_) => panic!("No folder /pastes, and couldn't create it"),
        }
    }

    HttpServer::new(|| 
        App::new()
            .route("/", web::get().to(index))
            .route("/", web::post().to(receive_form))
            .route("/{filename}", web::get().to(file))
        )
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
