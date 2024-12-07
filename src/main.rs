mod embedding;
mod routes;

use std::{sync::Arc, time::Duration};

use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use anyhow::Error;
use dim::{self, prompt::load_prompts};

use embedding::InMemoryVectorStore;
use log::info;
use tokio::{self, sync::Mutex};

pub struct SharedStores {
    pub clothes: Arc<Mutex<InMemoryVectorStore>>,
    pub face: Arc<Mutex<InMemoryVectorStore>>,
}

// Helper function to create a test vector store
pub fn initialize_clothes_store() -> InMemoryVectorStore {
    let prompts: Vec<String> =
        load_prompts("/Users/xinyubao/Documents/aesthetic-prototype/prompts_clothes").unwrap();

    InMemoryVectorStore::new(30, vec![], prompts, 2)
}

pub fn initialize_face_store() -> InMemoryVectorStore {
    let prompts: Vec<String> =
        load_prompts("/Users/xinyubao/Documents/aesthetic-prototype/prompts").unwrap();

    InMemoryVectorStore::new(30, vec![], prompts, 2)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // initiate a logger
    simple_logger::SimpleLogger::new().env().init().unwrap();

    // initialize vector stores
    let clothes_store = initialize_clothes_store();
    let face_store = initialize_face_store();

    // share it between threads
    let shared_clothes_store = Arc::new(Mutex::new(clothes_store));
    let shared_face_store = Arc::new(Mutex::new(face_store));
    let shared_store = Arc::new(Mutex::new(SharedStores {
        clothes: shared_clothes_store,
        face: shared_face_store,
    }));

    info!("In-Memory vector store is initialized.");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(shared_store.clone()))
            .configure(routes::config)
    })
    .client_request_timeout(Duration::from_secs(0))
    .client_disconnect_timeout(Duration::from_secs(0))
    .max_connection_rate(256)
    .bind(("0.0.0.0".to_string(), 9500))?
    .run()
    .await?;

    Ok(())
}
