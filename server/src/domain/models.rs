use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Person {
    pub id: Uuid,
    pub embeddings: Vec<Vec<f32>>,
}
