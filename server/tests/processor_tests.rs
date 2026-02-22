use std::error::Error as StdError;
use std::fs;
use std::path::PathBuf;

use zupeload::services::processor::process_images_and_tag;

const FACE_DETECTOR_MODEL: &str = "models/seeta_fd_frontal_v1.0.bin"; // Path expected but ignored by detector.rs
const EMBEDDING_MODEL: &str = "models/arcface.onnx";
const TEST_IMAGES_DIR: &str = "test_images";
const DB_PATH: &str = "target/test_tmp/processor_test.redb";
const SIMILARITY_THRESHOLD: f32 = 0.85;

#[test]
fn test_process_images_and_tag() -> Result<(), Box<dyn StdError>> {
    // Setup: Ensure temp directory exists
    let temp_dir = PathBuf::from("target/test_tmp");
    if !temp_dir.exists() {
        fs::create_dir_all(&temp_dir)?;
    }

    // Clean up previous test database
    if PathBuf::from(DB_PATH).exists() {
        fs::remove_file(DB_PATH)?;
    }

    println!("Running process_images_and_tag on {}...", TEST_IMAGES_DIR);

    let results = process_images_and_tag(
        TEST_IMAGES_DIR,
        FACE_DETECTOR_MODEL,
        EMBEDDING_MODEL,
        DB_PATH,
        SIMILARITY_THRESHOLD,
    )?;

    let total_images = fs::read_dir(TEST_IMAGES_DIR)?.filter(|e| e.is_ok()).count();
    println!(
        "Processed {} images and found {} faces.",
        total_images,
        results.len()
    );

    // Basic assertions
    assert!(
        !results.is_empty(),
        "Should have detected at least one face in the test images"
    );

    let mut person_map: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();
    for face in &results {
        println!(
            "  Detected face in {}: Person ID {}, Confidence {:.2}",
            face.image_path, face.person_id, face.confidence
        );
        assert!(!face.person_id.is_empty(), "Person ID should not be empty");
        assert!(face.confidence > 0.0, "Confidence should be greater than 0");

        person_map
            .entry(face.person_id.clone())
            .or_default()
            .push(face.image_path.clone());
    }

    println!("\n--- Matching Summary ---");
    println!("Total faces detected: {}", results.len());
    println!("Total unique people: {}", person_map.len());

    for (person_id, images) in &person_map {
        if images.len() > 1 {
            println!(
                "  Person {} matched in {} images: {:?}",
                person_id,
                images.len(),
                images
            );
        }
    }

    // With our test images (15 total, including many duplicates),
    // we expect significantly fewer unique people than total faces.
    assert!(
        person_map.len() < results.len(),
        "Matching logic failed: unique people count ({}) should be less than total faces ({}) because of duplicate images",
        person_map.len(),
        results.len()
    );

    // Verify database was created
    assert!(
        PathBuf::from(DB_PATH).exists(),
        "Database file should have been created"
    );

    Ok(())
}
