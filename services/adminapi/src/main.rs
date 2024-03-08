use actix_web::{
    delete, error, get,
    http::{header::ContentType, StatusCode},
    post,
    web::{self, delete},
    App, HttpResponse, HttpServer, Responder,
};
use derive_more::{Display, Error};
use qdrant_client::qdrant::vectors_config::Config;
use qdrant_client::qdrant::{CreateCollection, VectorParams, VectorsConfig};
use qdrant_client::{prelude::*, qdrant};

// consts go here
const QDRANT_URL_LOCAL: &str = "http://localhost:6334";

// structs
struct AppState {
    qdrant: QdrantClient,
}

// error handler for qdrant
#[derive(Debug, Display, Error)]
enum MyError {
    #[display(fmt = "qdrant error")]
    InternalError,

    #[display(fmt = "bad request")]
    BadClientData,
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
        }
    }
}

#[get("/health")]
async fn healthcheck() -> impl Responder {
    HttpResponse::Ok()
}

#[post("/create-collection")]
async fn create_collection(
    data: web::Data<AppState>,
    name: String,
) -> Result<impl Responder, MyError> {
    let client = &data.qdrant;

    let req = client
        .create_collection(&CreateCollection {
            collection_name: name.into(),
            vectors_config: Some(VectorsConfig {
                config: Some(Config::Params(VectorParams {
                    size: 10,
                    distance: Distance::Cosine.into(),
                    ..Default::default()
                })),
            }),
            ..Default::default()
        })
        .await;

    match req.into() {
        Ok(qdrant::CollectionOperationResponse { result, time }) => {
            Ok(HttpResponse::Ok().body("created successfully"))
        }
        Err(err) => Err(MyError::InternalError),
    }
}

#[delete("/delete-collection")]
async fn delete_collection(
    data: web::Data<AppState>,
    name: String,
) -> Result<impl Responder, MyError> {
    let client = &data.qdrant;

    let req = client.delete_collection(name).await;

    match req.into() {
        Ok(qdrant::CollectionOperationResponse { result, time }) => {
            Ok(HttpResponse::Ok().body("deleted successfully"))
        }
        Err(err) => Err(MyError::InternalError),
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
            }))
            .service(healthcheck)
            .service(create_collection)
            .service(delete_collection)
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}
