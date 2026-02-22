use anyhow::{Context, Result};
use std::fs;
use std::path::Path;
use tracing::{info, warn};
use uuid::Uuid;

use crate::domain::models::{DetectedFace, Person};
use crate::infrastructure::{database, detector, embeddings};

pub fn process_images_and_tag(
    images_dir: &str,
    detector_model: &str,
    embedder_model: &str,
    db_path: &str,
    threshold: f32,
) -> Result<Vec<DetectedFace>> {
    // Init components
    let detector = detector::FaceDetector::new(detector_model)
        .context("Failed to initialize FaceDetector")?;
    let mut embedder = embeddings::EmbeddingModel::new(embedder_model)
        .context("Failed to initialize EmbeddingModel")?;
    let store = database::PersonStore::new(db_path)
        .context("Failed to initialize PersonStore")?;

    let mut results: Vec<DetectedFace> = Vec::new();

    // Load all images from directory
    let entries = fs::read_dir(images_dir)
        .with_context(|| format!("Failed to read directory: {}", images_dir))?;

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !is_image(&path) {
            continue;
        }

        let path_str = path.to_string_lossy().to_string();
        info!("Processing: {}", path_str);

        let img = image::open(&path).with_context(|| format!("Failed to open image: {}", path_str))?;
        let faces = detector.detect(&img);

        info!("  Found {} face(s)", faces.len());

        for (face_crop, bbox, confidence) in faces {
            // Generate embedding for this face
            let embedding = match embedder.generate(&face_crop) {
                Ok(e) => e,
                Err(e) => {
                    warn!("Embedding failed: {}", e);
                    continue;
                }
            };

            // Try to match against known people
            let person_id = match store.find_match(&embedding, threshold)? {
                Some(existing) => {
                    info!("  Matched existing person: {}", existing.id);
                    // Add this embedding variant to improve future matching
                    // Only update the profile if the detection is high quality to prevent feature drift
                    if confidence >= 0.90 {
                        store.add_embedding(&existing.id, embedding)?;
                    }
                    existing.id.to_string()
                }
                None => {
                    // New person — create and store
                    let new_id_uuid = Uuid::new_v4();
                    let new_id = new_id_uuid.to_string();
                    if confidence >= 0.90 {
                        let person = Person {
                            id: new_id_uuid,
                            embeddings: vec![embedding],
                        };
                        store.save(&person)?;
                        info!("  New person created: {}", new_id);
                    } else {
                        info!("  Low-confidence face, not saving to DB but assigning temp ID: {}", new_id);
                    }
                    new_id
                }
            };

            results.push(DetectedFace {
                image_path: path_str.clone(),
                person_id,
                confidence,
                bbox,
            });
        }
    }

    Ok(results)
}

fn is_image(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("jpg") | Some("jpeg") | Some("png") | Some("webp")
    )
}
