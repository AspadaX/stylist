mod embedding;
mod routes;

use dim::{self, prompt::load_prompts};
use embedding::{InMemoryVectorStore, VectorStore};
use image::{DynamicImage, ImageBuffer};
use tokio;

// Helper function to create a test image
fn create_test_image() -> DynamicImage {
    let img_buffer = ImageBuffer::from_fn(100, 100, |_, _| {
        image::Rgba([255, 255, 255, 255])
    });
    DynamicImage::ImageRgba8(img_buffer)
}

// Helper function to create a test vector store
fn create_test_store() -> InMemoryVectorStore {
    let prompts = load_prompts(
        "/Users/xinyubao/Documents/aesthetic-prototype/prompts"
    ).unwrap();
    
    InMemoryVectorStore::new(
        30, 
        vec![],
        prompts,
        2,
    )
}

#[tokio::main]
async fn main() {
    let mut store = create_test_store();
    let test_image = create_test_image();
    
    // Test add
    println!("Trying vectorzation...");
    let result = store.add(
        "test_image",
        vec!["test description".to_string()],
        test_image.clone(),
    ).await;
    assert!(result.is_ok());
    println!("Vectorization: {:?}", result);
}
