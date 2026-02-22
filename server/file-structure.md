# Zupeload Server Project Structure

This project follows a layered architecture to separate concerns, improve maintainability, and follow Rust best practices.

## Directory Layout

```text
server/
├── Cargo.toml          # Project dependencies and configuration
├── src/
│   ├── lib.rs          # Library entry point and public API re-exports
│   ├── bin/
│   │   └── runner.rs   # Main binary entry point
│   ├── core/           # Foundational logic (non-business specific)
│   │   ├── mod.rs      # Core module definitions
│   │   ├── config.rs   # Configuration handling (env vars, etc.)
│   │   ├── logger.rs   # Logging initialization
│   │   └── utils.rs    # Generic utility functions (e.g., math)
│   ├── domain/         # Business logic entities and traits
│   │   ├── mod.rs      # Domain module definitions
│   │   └── models.rs   # Data structures (Person, BoundingBox, etc.)
│   ├── infrastructure/ # External system integrations
│   │   ├── mod.rs      # Infrastructure module definitions
│   │   ├── database.rs # Persistent storage (redb)
│   │   ├── detector.rs # Face detection engine (rust-faces)
│   │   └── embeddings.rs # AI embedding generation (ort/ONNX)
│   └── services/       # High-level orchestration and use cases
│       ├── mod.rs      # Services module definitions
│       └── processor.rs # Main image processing and tagging pipeline
└── tests/
    └── database_tests.rs # Integration tests
```

## Architectural Layers

### Core
Contains foundational components that are independent of the business domain. This includes configuration management, logging, and mathematical utilities like `cosine_similarity`.

### Domain
Defines the core data models and business logic. This layer should be as "pure" as possible, containing the data structures used throughout the application.

### Infrastructure
Handles integrations with external systems, libraries, and databases. This is where the concrete implementations for face detection, embedding generation, and database access reside.

### Services
Orchestrates different components from the infrastructure and domain layers to fulfill specific use cases. The `processor` service, for example, coordinates detection, embedding, and storage.

## Best Practices Followed
- **Module Separation**: Components are grouped by their responsibility.
- **Dependency Inversion**: High-level services depend on abstractions (via re-exports in `lib.rs`) rather than being tightly coupled to internal file structures.
- **Clean Public API**: `lib.rs` re-exports only what is necessary for external users (like the binary or tests), keeping internal implementation details hidden.
- **Utility Extraction**: Mathematical and generic logic is extracted from business logic into `core/utils.rs`.
- **Idiomatic Naming**: Follows standard Rust naming conventions and module structures.
