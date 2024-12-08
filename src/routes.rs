use std::sync::Arc;

use actix_web::{
    delete, get, post,
    web::{self, Data, Json},
    HttpResponse, Responder,
};
use anyhow::Error;
use base64;
use image::{load_from_memory, DynamicImage};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

use crate::{embedding::VectorStore, SharedStores};

/// Decodes a base64 encoded image string into a DynamicImage
/// 
/// # Arguments
/// * `b64_str` - Base64 encoded string of the image
/// 
/// # Returns
/// * `Result<DynamicImage, Error>` - The decoded image or an error
pub fn decode_base64_image(b64_str: &str) -> Result<DynamicImage, Error> {
    let decoded_bytes: Vec<u8> = base64::decode(b64_str)?;
    let img: DynamicImage = load_from_memory(&decoded_bytes)?;
    Ok(img)
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Gender {
    Male,
    Female,
}

/// Request structure for uploading images
#[derive(Debug, Serialize, Deserialize)]
pub struct ImageUploadRequest {
    pub name: String,
    pub gender: Gender,
    pub image: String, // in base64
}

/// Example:
/// ```json
/// {
///     "name": "Blue T-shirt",
///     "gender": "Male",
///     "image": "base64_encoded_image_string"
/// }
/// ```

#[derive(Serialize, Deserialize)]
struct ImageUploadResponse {
    id: String,
    success: bool,
}

/// Request structure for similarity search
#[derive(Deserialize)]
struct SimilarityRequest {
    user_image: String,
    top_n: usize,
}

/// Example:
/// ```json
/// {
///     "user_image": "base64_encoded_image_string",
///     "top_n": 5
/// }
/// ```

#[derive(Debug, Deserialize, Serialize)]
pub struct BasicResponse<T: Serialize> {
    pub status: bool,
    pub message: String,
    pub data: Option<T>,
}

/// Upload a new piece of clothing
/// 
/// # HTTP Request
/// POST /api/clothes/upload
/// 
/// # Request Body
/// JSON object containing name, gender and base64 encoded image
#[post("/api/clothes/upload")]
async fn upload_clothes(
    shared_stores: Data<Arc<Mutex<SharedStores>>>,
    request: Json<ImageUploadRequest>,
) -> impl Responder {
    info!(
        "Received upload request for clothes with name: {}",
        request.name
    );

    let shared_stores = shared_stores.lock().await;
    let mut clothes_store = shared_stores.clothes.lock().await;

    match decode_base64_image(&request.image) {
        Ok(result) => {
            match clothes_store
                .add(&request.name, vec!["".to_string()], result)
                .await
            {
                Ok(_) => {
                    info!("Successfully added clothes: {}", request.name);
                    HttpResponse::Ok().json(BasicResponse::<String> {
                        status: true,
                        message: "Clothes added successfully.".to_string(),
                        data: None,
                    })
                }
                Err(error) => {
                    error!("Failed to add clothes to vector store: {}", error);
                    HttpResponse::InternalServerError().json(BasicResponse::<String> {
                        status: false,
                        message: error.to_string(),
                        data: None,
                    })
                }
            }
        }
        Err(error) => {
            error!("Failed to decode base64 image: {}", error);
            HttpResponse::BadRequest().json(BasicResponse::<String> {
                status: false,
                message: error.to_string(),
                data: None,
            })
        }
    }
}

/// Get all clothes
/// 
/// # HTTP Request
/// GET /api/clothes/get
#[get("/api/clothes/get")]
async fn get_clothes(shared_stores: Data<Arc<Mutex<SharedStores>>>) -> impl Responder {
    info!("Handling request to get all clothes");
    let shared_stores = shared_stores.lock().await;
    let clothes_store = shared_stores.clothes.lock().await;
    HttpResponse::Ok().json(clothes_store.get_all())
}

/// Delete a piece of clothing by ID
/// 
/// # HTTP Request
/// DELETE /api/clothes/delete/{id}
/// 
/// # URL Parameters
/// * `id` - The ID of the clothing item to delete
#[delete("/api/clothes/delete/{id}")]
async fn delete_clothes(
    id: web::Path<String>,
    shared_stores: Data<Arc<Mutex<SharedStores>>>,
) -> impl Responder {
    info!("Received delete request for clothes id: {}", id);
    let shared_stores = shared_stores.lock().await;
    let mut clothes_store = shared_stores.clothes.lock().await;

    match id.parse::<usize>() {
        Ok(id) => match clothes_store.delete(id).await {
            Ok(_) => {
                info!("Successfully deleted clothes with id: {}", id);
                HttpResponse::Ok().json(BasicResponse::<String> {
                    status: true,
                    message: "Clothes deleted successfully".to_string(),
                    data: None,
                })
            }
            Err(e) => {
                error!("Failed to delete clothes with id {}: {}", id, e);
                HttpResponse::NotFound().json(BasicResponse::<String> {
                    status: false,
                    message: format!("Failed to delete clothes: {}", e),
                    data: None,
                })
            }
        },
        Err(_) => {
            warn!("Invalid ID format provided: {}", id);
            HttpResponse::BadRequest().json(BasicResponse::<String> {
                status: false,
                message: "Invalid ID format".to_string(),
                data: None,
            })
        }
    }
}

/// Calculate similarity between uploaded image and stored clothes
/// 
/// # HTTP Request
/// POST /api/similarity/calculate
/// 
/// # Request Body
/// JSON object containing base64 encoded image and number of results to return
#[post("")]
async fn calculate_similarity(
    shared_stores: Data<Arc<Mutex<SharedStores>>>,
    request: web::Json<SimilarityRequest>,
) -> impl Responder {
    info!(
        "Processing similarity calculation request for top_n: {}",
        request.top_n
    );
    let shared_stores = shared_stores.lock().await;
    let clothes_store = shared_stores.clothes.lock().await;

    match decode_base64_image(&request.user_image) {
        Ok(image) => match clothes_store.search(image, request.top_n).await {
            Ok(results) => {
                info!("Successfully completed similarity search");
                HttpResponse::Ok().json(BasicResponse {
                    status: true,
                    message: "Search operation succeeded.".to_string(),
                    data: Some(results),
                })
            }
            Err(e) => {
                error!("Error during similarity search: {}", e);
                HttpResponse::InternalServerError().json(BasicResponse::<String> {
                    status: false,
                    message: format!("Error searching similar images: {}", e),
                    data: None,
                })
            }
        },
        Err(e) => {
            error!("Failed to decode uploaded image: {}", e);
            HttpResponse::BadRequest().json(BasicResponse::<String> {
                status: false,
                message: format!("Failed to decode image: {}", e),
                data: None,
            })
        }
    }
}

/// Save the vector stores to disk
/// 
/// # HTTP Request
/// GET /api/store/save
/// 
/// # Request Body
/// Empty
#[get("/api/store/save")]
async fn save_store(shared_stores: Data<Arc<Mutex<SharedStores>>>) -> impl Responder {
    info!("Handling request to save stores to disk");
    let shared_stores = shared_stores.lock().await;

    match shared_stores.save("vector_stores.json").await {
        Ok(_) => {
            info!("Successfully saved vector stores to disk");
            HttpResponse::Ok().json(BasicResponse::<String> {
                status: true,
                message: "Vector stores saved successfully".to_string(),
                data: None,
            })
        }
        Err(e) => {
            error!("Failed to save vector stores: {}", e);
            HttpResponse::InternalServerError().json(BasicResponse::<String> {
                status: false,
                message: format!("Failed to save vector stores: {}", e),
                data: None,
            })
        }
    }
}

/// Load the vector stores from disk
/// 
/// # HTTP Request
/// GET /api/store/load
/// 
/// # Request Body
/// Empty
#[get("/api/store/load")]
async fn load_store(shared_stores: Data<Arc<Mutex<SharedStores>>>) -> impl Responder {
    info!("Handling request to load stores from disk");
    let shared_stores = shared_stores.lock().await;

    match shared_stores.load("vector_stores.json").await {
        Ok(_) => {
            info!("Successfully loaded vector stores from disk");
            HttpResponse::Ok().json(BasicResponse::<String> {
                status: true,
                message: "Vector stores loaded successfully".to_string(),
                data: None,
            })
        }
        Err(e) => {
            error!("Failed to load vector stores: {}", e);
            HttpResponse::InternalServerError().json(BasicResponse::<String> {
                status: false,
                message: format!("Failed to load vector stores: {}", e),
                data: None,
            })
        }
    }
}

pub fn config(cfg: &mut web::ServiceConfig) {
    cfg.service(upload_clothes)
        .service(get_clothes)
        .service(delete_clothes)
        .service(calculate_similarity)
        .service(save_store)
        .service(load_store);
}
