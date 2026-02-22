use std::error::Error as StdError;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use zupeload::infrastructure::detector::FaceDetector;
use zupeload::infrastructure::embeddings::EmbeddingModel;
use zupeload::infrastructure::database::{PersonStore, load_all_people};
use zupeload::domain::models::{Person, DetectedFace};
use zupeload::core::utils::cosine_similarity;

// Helper function to check if a path is an image
fn is_image(path: &PathBuf) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("jpg") | Some("jpeg") | Some("png") | Some("webp")
    )
}

// Model and directory constants
const FACE_DETECTOR_MODEL: &str = "models/seeta_fd_frontal_v1.0.bin";
const EMBEDDING_MODEL: &str = "models/arcface.onnx";
const TEST_IMAGES_DIR: &str = "test_images";
const DB_PATH_SUFFIX: &str = "test_face_matching.redb";
const SIMILARITY_THRESHOLD: f32 = 0.65;

#[test]
fn test_face_detection_and_matching() -> Result<(), Box<dyn StdError>> {
    // Setup: Temporary directory and database
    let temp_dir_path = PathBuf::from("target/test_tmp"); 
    if !temp_dir_path.exists() {
        fs::create_dir_all(&temp_dir_path).expect("Failed to create temp directory");
    }
    let db_path = temp_dir_path.join(DB_PATH_SUFFIX);

    // Clean up previous test runs
    if db_path.exists() {
        std::fs::remove_file(&db_path)?;
    }

    let image_dir = PathBuf::from(TEST_IMAGES_DIR);
    if !image_dir.exists() {
        fs::create_dir_all(&image_dir)?;
    }

    // Initialize components
    let person_store_db_path = temp_dir_path.join("test_face_store.redb");
    if person_store_db_path.exists() {
        std::fs::remove_file(&person_store_db_path)?;
    }
    
    let store = PersonStore::new(person_store_db_path.to_str().ok_or("Invalid path")?)?;
    let detector = FaceDetector::new(FACE_DETECTOR_MODEL)?;
    let mut embedder = EmbeddingModel::new(EMBEDDING_MODEL)?;

    let mut detected_faces_results: Vec<DetectedFace> = Vec::new();
    let mut total_faces_processed_for_embedding = 0; 
    let mut new_people_created_count = 0;

    println!("Starting face detection and matching test...");

    // Load and process images from the test directory
    for entry in fs::read_dir(&image_dir)? {
        let entry = entry?;
        let path = entry.path();
        if !is_image(&path) {
            continue;
        }

        let path_str = path.to_string_lossy().to_string();
        println!("  Processing image: {}", path_str);

        let img = image::open(&path)?;
        println!("    Image dimensions: {}x{}", img.width(), img.height());
        let faces_data = detector.detect(&img);

        println!("    Found {} face(s) in {}", faces_data.len(), path_str);

        for (face_crop, bbox, confidence) in faces_data {
            // Generate embedding for this face
            let embedding = match embedder.generate(&face_crop) {
                Ok(e) => e,
                Err(e) => {
                    eprintln!("    Embedding failed for face in {:?}: {}", path, e);
                    continue; 
                }
            };
            total_faces_processed_for_embedding += 1;

            // Try to match against known people
            let all_stored_people = load_all_people(&store)?; 
            let mut matched_person_id: Option<Uuid> = None; 

            for stored_person in all_stored_people {
                if stored_person.embeddings.iter().any(|emb| {
                    cosine_similarity(emb, &embedding) > SIMILARITY_THRESHOLD
                }) {
                    matched_person_id = Some(stored_person.id);
                    println!("    Matched existing person: {}", stored_person.id);
                    break; 
                }
            }

            let person_id_for_result = match matched_person_id {
                Some(id) => id,
                None => {
                    // New person — create and store
                    let new_id = Uuid::new_v4();
                    let person = Person {
                        id: new_id,
                        embeddings: vec![embedding],
                    };
                    store.save(&person)?;
                    println!("    New person created: {}", new_id);
                    new_people_created_count += 1; 
                    new_id 
                }
            };
            
            detected_faces_results.push(DetectedFace {
                image_path: path_str.clone(),
                person_id: person_id_for_result.to_string(), 
                confidence,
                bbox,
            });
        }
    }

    // --- Assertions ---
    println!("
=== Test Summary ===");
    println!("Total faces processed for embedding: {}", total_faces_processed_for_embedding);
    println!("Total new people created from scratch: {}", new_people_created_count);

    let final_stored_people = load_all_people(&store)?; 
    let final_unique_people_count = final_stored_people.len();
    println!("Total unique people found in store after processing: {}", final_unique_people_count);

    assert_eq!(
        new_people_created_count,
        final_unique_people_count,
        "Mismatch: Number of newly created people ({}) does not match the total unique people in the store ({}).",
        new_people_created_count,
        final_unique_people_count
    );

    assert_eq!(
        total_faces_processed_for_embedding,
        detected_faces_results.len(),
        "Mismatch: Total faces processed for embedding ({}) does not match the number of recorded detected faces ({}).",
        total_faces_processed_for_embedding,
        detected_faces_results.len()
    );

    for person in &final_stored_people {
        assert!(!person.embeddings.is_empty(), "Stored person {} has no embeddings.", person.id);
    }

    println!("Test 'test_face_detection_and_matching' completed successfully.");

    Ok(())
}
