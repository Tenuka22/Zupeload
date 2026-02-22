pub mod core;
pub mod domain;
pub mod infrastructure;
pub mod services;

// Re-export common types for easier access
pub use crate::core::config::Config;
pub use crate::core::logger;
pub use crate::domain::models::{DetectedFace, Person, BoundingBox};
pub use crate::services::processor::process_images_and_tag;
