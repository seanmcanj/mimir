use actix_web::{
    delete, error, get,
    http::{header::ContentType, StatusCode},
    post,
    web::{self},
    App, HttpResponse, HttpServer, Responder,
};
use derive_more::{Display, Error};
use qdrant_client::qdrant::{vectors_config::Config, Condition, Filter};
use qdrant_client::qdrant::{CreateCollection, VectorParams, VectorsConfig};
use qdrant_client::{prelude::*, qdrant};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Error, ErrorKind};


// consts go here
const QDRANT_URL_LOCAL: &str = "http://localhost:6334";

// structs
struct AppState {
    qdrant: QdrantClient,
    httpclient: reqwest::Client,
}

#[derive(Serialize, Deserialize, Debug)]
struct EmbedResponse {
    status: String,
    embeddings: Box<[f32]>,
}

// error handler for qdrant
#[derive(Debug, Display, Error)]
enum MyError {
    #[display(fmt = "qdrant error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,

    #[display(fmt = "unknown error")]
    UnknownError,
}

impl error::ResponseError for MyError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            MyError::InternalError => StatusCode::INTERNAL_SERVER_ERROR,
            MyError::BadClientData => StatusCode::BAD_REQUEST,
            MyError::UnknownError => StatusCode::FAILED_DEPENDENCY,
        }
    }
}

#[get("/health")]
async fn healthcheck() -> impl Responder {
    HttpResponse::Ok()
}

#[post("/query")]
async fn search_query(
    data: web::Data<AppState>,
    tenantid: String,
    query: String,
) -> Result<impl Responder, MyError> {
    let client = &data.qdrant;
    let httpclient = &data.httpclient;
    let mut headers: HashMap<String, String> = HashMap::new();
    headers.insert("Connection".to_string(), "close".to_string());

    let mut map = HashMap::new();
    map.insert("docs", [query]);

    let resp = httpclient
        .post("http://localhost:3000/get-embeddings")
        .json(&map)
        .send()
        .await
        .unwrap();



    match resp.status() {
        reqwest::StatusCode::OK => match resp.text().await {
            Ok(parsed) => {
                let embeddings: serde_json::Value = serde_json::from_str(&parsed).unwrap();
                println!("{:?}", embeddings);
                Ok(HttpResponse::Ok().body("created successfully"))
            },
            Err(err) => {
                Err(MyError::UnknownError)
            },
        },
        other => Err(MyError::UnknownError),
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let qdrant_url = std::env::var("QDRANT_URL")
        .unwrap_or(QDRANT_URL_LOCAL.to_string())
        .to_string();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(AppState {
                qdrant: QdrantClient::from_url(&qdrant_url).build().unwrap(),
                httpclient: reqwest::Client::new(),
            }))
            .service(healthcheck)
            .service(search_query)
    })
    .bind(("0.0.0.0", 4000))?
    .run()
    .await
}
