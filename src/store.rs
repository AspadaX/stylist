use std::{fs::File, io::{BufReader, BufWriter}, sync::Arc};

use crate::embedding::InMemoryVectorStore;
use anyhow::Error;
use serde::{Deserialize, Serialize};
use tokio::{self, sync::Mutex};

#[derive(Debug, Clone)]
pub struct SharedStores {
    pub clothes: Arc<Mutex<InMemoryVectorStore>>,
    pub face: Arc<Mutex<InMemoryVectorStore>>,
}

/// for persistant storage
#[derive(Serialize, Deserialize)]
struct PersistentStores {
    clothes: InMemoryVectorStore,
    face: InMemoryVectorStore,
}

impl SharedStores {
    // Save both stores to disk
    pub async fn save(&self, path: &str) -> Result<(), Error> {
        let clothes = self.clothes.lock().await;
        let face = self.face.lock().await;

        let data = PersistentStores {
            clothes: clothes.clone(),
            face: face.clone(),
        };

        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &data)?;
        Ok(())
    }

    // Load both stores from disk
    pub async fn load(&self, path: &str) -> Result<(), Error> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        let data: PersistentStores = serde_json::from_reader(reader)?;

        let mut clothes = self.clothes.lock().await;
        let mut face = self.face.lock().await;

        *clothes = data.clothes;
        *face = data.face;

        Ok(())
    }
}
