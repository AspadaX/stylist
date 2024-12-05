use std::fmt::Display;

use anyhow::{Error, Ok, Result};
use async_openai::{config::OpenAIConfig, Client};
use dim::{
    llm::instantiate_client,
    vector::{self, Vector},
    vectorizations::vectorize_image_concurrently,
};
use image::DynamicImage;

/// Error variants related to DataEntry operations
#[derive(Debug, Clone, Copy)]
pub enum DataEntryErrors {
    /// Indicates that no data entry was found for the given criteria
    NoDataWasFound,
}

impl std::error::Error for DataEntryErrors {}

impl Display for DataEntryErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoDataWasFound => write!(f, "No data entry was found!"),
        }
    }
}

/// Represents a single data entry in the vector store
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct DataEntry {
    /// Unique identifier for the data entry
    pub id: usize,
    /// Name associated with the data entry
    pub name: String,
    /// Vector representation of the data
    pub vector: Vec<f64>,
    /// List of descriptions associated with the data
    pub descriptions: Vec<String>,
}

/// Defines essential operations that must be implemented by vector stores
pub trait VectorStore {
    /// Search for similar entries given an image
    ///
    /// # Arguments
    /// * `image` - The image to search for similar entries
    /// * `top_n` - Number of most similar entries to return
    async fn search(&self, image: DynamicImage, top_n: usize) -> Result<Vec<DataEntry>, Error>;

    /// Add a new entry to the vector store
    ///
    /// # Arguments
    /// * `name` - Name of the entry
    /// * `descriptions` - List of descriptions for the entry  
    /// * `image` - Image to store
    async fn add(
        &mut self,
        name: &str,
        descriptions: Vec<String>,
        image: DynamicImage,
    ) -> Result<()>;

    /// Delete an entry from the store by ID
    ///
    /// # Arguments
    /// * `id` - ID of the entry to delete
    async fn delete(&mut self, id: usize) -> Result<()>;

    /// Edit an existing entry with new data
    ///
    /// # Arguments
    /// * `image` - New image
    /// * `data_entry` - Updated data entry
    async fn edit(&mut self, image: DynamicImage, data_entry: DataEntry) -> Result<()>;
}

/// In-memory implementation of a vector store
pub struct InMemoryVectorStore {
    /// Storage for data entry metadata
    data_entries: Vec<DataEntry>,
    /// Annotations used for prompting
    prompt_annotations: Vec<String>,
    /// Prompts used for vectorization
    prompts: Vec<String>,
    /// Size of prompts to use
    prompt_size: usize,
    /// Dimension of the vectors
    dimensions: usize
}

impl InMemoryVectorStore {
    /// Create a new InMemoryVectorStore instance
    ///
    /// # Arguments
    /// * `dimensions` - Dimensionality of vectors
    /// * `prompt_annotations` - Annotations for prompts
    /// * `prompts` - Prompts for vectorization
    /// * `prompt_size` - Size of prompts to use
    pub fn new(
        dimensions: usize,
        prompt_annotations: Vec<String>,
        prompts: Vec<String>,
        prompt_size: usize,
    ) -> Self {
        Self {
            data_entries: Vec::new(),
            prompts: prompts,
            prompt_size: prompt_size,
            prompt_annotations: prompt_annotations,
            dimensions: dimensions
        }
    }

    /// Store entry metadata in key-value storage
    ///
    /// # Arguments
    /// * `name` - Name of the entry
    /// * `descriptions` - Descriptions for the entry
    /// * `vector` - Vector representation
    ///
    /// # Returns
    /// ID of the stored entry
    fn kv_storage(
        &mut self,
        name: &str,
        descriptions: Vec<String>,
        vector: Vec<f64>,
    ) -> Result<usize, Error> {
        let current_id: usize = self.data_entries.len() + 1;

        self.data_entries.push(DataEntry {
            id: current_id,
            name: name.to_string(),
            vector: vector,
            descriptions: descriptions,
        });

        Ok(current_id)
    }

    /// Retrieve entry metadata by ID
    ///
    /// # Arguments
    /// * `id` - ID of entry to retrieve
    fn kv_search(&self, query_vector: Vec<f64>, top_n: usize) -> Result<Vec<DataEntry>, Error> {
        if self.data_entries.is_empty() {
            return Err(DataEntryErrors::NoDataWasFound.into());
        }

        // Calculate similarities and store with indices
        let mut similarities: Vec<(usize, f64)> = self.data_entries
            .iter()
            .enumerate()
            .map(|(idx, entry)| (
                idx,
                self.cosine_similarity(&query_vector, &entry.vector)
            ))
            .collect();

        // Sort by similarity score in descending order
        similarities.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

        // Take top n entries
        let top_entries: Vec<DataEntry> = similarities
            .into_iter()
            .take(top_n)
            .map(|(idx, _)| self.data_entries[idx].clone())
            .collect();

        if top_entries.is_empty() {
            return Err(DataEntryErrors::NoDataWasFound.into());
        }

        Ok(top_entries)
    }
    
    // Helper function to calculate cosine similarity between two vectors
    fn cosine_similarity(&self, a: &[f64], b: &[f64]) -> f64 {
        let dot_product: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        let norm_a: f64 = a.iter().map(|x| x * x).sum::<f64>().sqrt();
        let norm_b: f64 = b.iter().map(|x| x * x).sum::<f64>().sqrt();
        
        if norm_a == 0.0 || norm_b == 0.0 {
            return 0.0;
        }
        
        dot_product / (norm_a * norm_b)
    }

    /// Delete entry metadata by ID
    ///
    /// # Arguments  
    /// * `id` - ID of entry to delete
    fn kv_delete(&mut self, id: usize) -> Result<(), Error> {
        // Find the position of the entry with matching id
        if let Some(index) = self
            .data_entries
            .iter()
            .position(|entry: &DataEntry| entry.id == id)
        {
            // Remove the entry and return Ok if found
            self.data_entries.remove(index);
            Ok(())
        } else {
            // Return error if no matching entry was found
            Err(DataEntryErrors::NoDataWasFound.into())
        }
    }

    /// Update entry metadata by ID
    ///
    /// # Arguments
    /// * `id` - ID of entry to update
    /// * `data_entry` - New data entry
    fn kv_edit(&mut self, id: usize, data_entry: DataEntry) -> Result<(), Error> {
        if let Some(index) = self.data_entries.iter().position(|entry| entry.id == id) {
            self.data_entries[index] = data_entry;
        } else {
            // Return error if no matching entry was found
            return Err(DataEntryErrors::NoDataWasFound.into());
        }

        Ok(())
    }
}

impl VectorStore for InMemoryVectorStore {
    async fn add(
        &mut self,
        name: &str,
        descriptions: Vec<String>,
        image: DynamicImage,
    ) -> Result<(), Error> {
        let client: Client<OpenAIConfig> = instantiate_client::<OpenAIConfig>(None)?;

        // initialize the vectorization mechanics
        let mut vector: vector::Vector<DynamicImage> = Vector::new(
            self.dimensions,
            self.prompt_annotations.clone(),
            self.prompts.clone(),
            self.prompt_size,
            image,
        );

        println!("Vectorizing...");
        vectorize_image_concurrently::<OpenAIConfig>(&mut vector, client).await?;

        println!("Try getting vectors...");
        let new_vector: Vec<f64> = vector.get_vector();
        println!("{:?}", &new_vector);

        // store the information to a kv storage, and get a corresponding
        // key for later retrieval.
        let _: usize = self.kv_storage(name, descriptions, new_vector.clone())?;

        Ok(())
    }

    async fn edit(&mut self, image: DynamicImage, data_entry: DataEntry) -> Result<(), Error> {
        // delete the original data entry first
        self.kv_delete(data_entry.id)?;

        // store the new data entry
        self.add(&data_entry.name, data_entry.descriptions, image)
            .await?;

        Ok(())
    }

    async fn delete(&mut self, id: usize) -> Result<()> {
        // delete both the vectors and the data entry
        self.kv_delete(id)?;

        Ok(())
    }

    async fn search(&self, image: DynamicImage, top_n: usize) -> Result<Vec<DataEntry>, Error> {
        let client: Client<OpenAIConfig> = instantiate_client::<OpenAIConfig>(None)?;

        // initialize the vectorization mechanics
        let mut vector: vector::Vector<DynamicImage> = Vector::new(
            self.dimensions,
            self.prompt_annotations.clone(),
            self.prompts.clone(),
            self.prompt_size,
            image,
        );

        vectorize_image_concurrently::<OpenAIConfig>(&mut vector, client).await?;

        let new_vector: Vec<f64> = vector.get_vector();

        let data_entries: Vec<DataEntry> = self.kv_search(
            new_vector, 
            top_n
        )?;

        Ok(data_entries)
    }
}
