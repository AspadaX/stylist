use stylist::embedding::*;

#[cfg(test)]
mod tests {
    use super::*;
    use dim::prompt::load_prompts;
    use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba};
    use tokio;

    // Helper function to create a test image
    fn create_test_image() -> DynamicImage {
        let img_buffer = ImageBuffer::from_fn(100, 100, |_, _| {
            Rgba([255, 255, 255, 255])
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

    #[test]
    fn test_data_entry_creation() {
        let entry = DataEntry {
            id: 1,
            name: "test".to_string(),
            vector: vec![0.1, 0.2, 0.3],
            descriptions: vec!["test desc".to_string()],
        };

        assert_eq!(entry.id, 1);
        assert_eq!(entry.name, "test");
        assert_eq!(entry.vector, vec![0.1, 0.2, 0.3]);
        assert_eq!(entry.descriptions, vec!["test desc"]);
    }

    #[test]
    fn test_data_entry_errors_display() {
        let error = DataEntryErrors::NoDataWasFound;
        assert_eq!(error.to_string(), "No data entry was found!");
    }

    #[tokio::test]
    async fn test_vector_store_crud_operations() {
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

        // Test search
        let search_results = store.search(test_image.clone(), 1).await;
        assert!(search_results.is_ok());
        let results = search_results.unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].name, "test_image");

        // Test delete
        let delete_result = store.delete(1).await;
        assert!(delete_result.is_ok());

        // Test search after delete
        let search_after_delete = store.search(test_image.clone(), 1).await;
        assert!(search_after_delete.is_ok());
        assert!(search_after_delete.unwrap().is_empty());
    }

}