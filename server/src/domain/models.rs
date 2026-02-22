use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Person {
    pub id: Uuid,
    pub embeddings: Vec<Vec<f32>>,
}

#[derive(Debug)]
pub struct DetectedFace {
    pub image_path: String,
    pub person_id: String,
    pub confidence: f64,
    pub bbox: BoundingBox,
}

#[derive(Debug)]
pub struct BoundingBox {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
