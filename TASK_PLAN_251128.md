# **Rigger RAG & Artifacts Implementation Plan**

Status: Planning  
Epic: Intelligent Context & Research Artifacts  
Architecture: Hexagonal (HEXSER)  
Dependencies: rig-core, sqlx, sqlite, sqlite-vec (Vector Search Extension)

## **Context**

We are adding a RAG (Retrieval-Augmented Generation) system to Rigger. This involves storing "Artifacts" (chunks of knowledge, PRD sections, researched facts, file summaries) in a SQL table with vector embeddings. Agents will use these artifacts to improve context during task generation and execution.

## **Architecture Overview**

### **Domain Layer (task_manager)**

* **Entity**: Artifact
  * id: UUID
  * project_id: Link to parent project
  * source_id: Link to source (PRD ID, File Path, or Task ID)
  * source_type: Enum (PRD, File, WebResearch, UserInput)
  * content: Text content
  * embedding: Vector data (Vec<f32>)
  * metadata: JSON (page numbers, file lines, urls)
  * created_at: Timestamp

### **Ports Layer (task_manager, task_orchestrator)**

* **Repository Port**: ArtifactRepositoryPort
  * save(artifact)
  * search_by_similarity(query_embedding, limit, threshold) -> Vec<Artifact>
  * find_by_source(source_id)
* **Service Port**: EmbeddingPort (in task_orchestrator)
  * generate_embedding(text) -> Vec<f32>
  * generate_embeddings(texts) -> Vec<Vec<f32>>

### **Adapters Layer**

* **Storage**: SqliteArtifactAdapter
  * Uses sqlx to interact with SQLite.
  * **Strategy (Vector Search)**: Utilize the sqlite-vec extension for high-performance, native vector search capabilities within SQLite. We will **not** implement naive cosine similarity in Rust. This leverages "Vector database capabilities" directly in the SQL engine (the "Rustica system" plugin approach).
  * *Mechanism*: Load the sqlite-vec extension at runtime on the SQLite connection pool. Use vec0 virtual tables or standard tables with vector functions for similarity search (vec_distance_cosine, etc.).
* **AI**: RigEmbeddingAdapter
  * Wraps rig-core's embedding functionality (using Ollama/OpenAI).

## **Implementation Phases**

### **Phase 1: Domain Modeling (Brain)**

Define the core structures and interfaces without implementation details.

* [ ] **Task 1.1**: Create task_manager/src/domain/artifact.rs
  * Define Artifact struct with HexEntity derive.
  * Define ArtifactType enum.
* [ ] **Task 1.2**: Create task_manager/src/ports/artifact_repository_port.rs
  * Define ArtifactRepositoryPort trait extending Repository \+ QueryRepository.
  * Add specialized method find_similar.
* [ ] **Task 1.3**: Create task_orchestrator/src/ports/embedding_port.rs
  * Define interface for generating embeddings from text.

### **Phase 2: Infrastructure & Persistence (Muscle)**

Implement the storage layer in SQLite using sqlite-vec.

* [ ] **Task 2.1**: Add sqlite-vec dependency and setup extension loading
  * Add sqlite-vec crate to Cargo.toml.
  * Update SqliteTaskAdapter::connect_and_init to load the extension via sqlx::sqlite::SqliteConnectOptions::extension.
* [ ] **Task 2.2**: Update Schema for Vector Support
  * Create artifacts table for metadata and content.
  * Create artifacts_vec virtual table (using vec0 module) for vector indexing if using sqlite-vec specific indexing, or standard columns if using scalar quantization functions.
  * Schema: id (TEXT PK), project_id (TEXT), embedding (FLOAT32[N]) (syntax depends on extension version).
* [ ] **Task 2.3**: Create task_manager/src/adapters/sqlite_artifact_adapter.rs
  * Implement Repository for Artifact.
  * Implement ArtifactRepositoryPort.
  * **Implementation**: Use sqlite-vec functions in SQL queries.  
    SELECT rowid, distance   
    FROM artifacts_vec   
    WHERE embedding MATCH ?   
    ORDER BY distance   
    LIMIT ?

### **Phase 3: AI Integration (Muscle)**

Implement the embedding generation using Rig.

* [ ] **Task 3.1**: Create task_orchestrator/src/adapters/rig_embedding_adapter.rs
  * Implement EmbeddingPort.
  * Use rig::embeddings::EmbeddingsBuilder.
  * Support switching providers via ProviderFactory.
* [ ] **Task 3.2**: Update task_orchestrator/src/adapters/provider_factory.rs
  * Add method create_embedding_adapter().

### **Phase 4: Integration & Ingestion (Flow)**

Wire it all together to ingest PRDs and create artifacts.

* [ ] **Task 4.1**: Create task_manager/src/domain/services/artifact_service.rs
  * Service to chunk text (simple paragraph/markdown splitting) and prepare Artifact entities.
* [ ] **Task 4.2**: Update RigPRDParserAdapter in task_orchestrator
  * Inject EmbeddingPort and ArtifactRepositoryPort into the adapter (or the Use Case calling it).
  * *Better Approach*: Keep Parser focused. Update rigger_cli/src/commands/parse.rs (Application Layer) to handle the flow:
    1. Parse PRD (get content).
    2. Call ArtifactService to chunk PRD content.
    3. Call EmbeddingPort to vectorize chunks.
    4. Save to ArtifactRepository.
    5. *Then* proceed with Task Generation, passing retrieved context if needed.

### **Phase 5: RAG Retrieval & Agent Usage**

Equip agents with the ability to recall information.

* [ ] **Task 5.1**: Create SearchArtifactsTool in task_orchestrator/src/tools/
  * Implements Rig's Tool trait.
  * Takes a query string.
  * Calls EmbeddingPort -> ArtifactRepository.find_similar.
* [ ] **Task 5.2**: Update RigPRDParserAdapter prompts
  * Inject relevant Artifacts into the system prompt context when generating tasks.
* [ ] **Task 5.3**: Update RigTaskDecompositionAdapter
  * Allow it to search artifacts to better understand "how" to decompose a task.

### **Phase 6: CLI Commands**

User-facing interactions.

* [ ] **Task 6.1**: Add rig artifacts list command.
* [ ] **Task 6.2**: Add rig artifacts search <query> command (debug tool to test RAG).
* [ ] **Task 6.3**: Add rig artifacts search to TUI spotlight.

## **Technical Guidelines**

1. **No use statements**: All paths must be fully qualified (e.g., task_manager::domain::artifact::Artifact).
2. **Single File Responsibility**: One struct/trait per file.
3. **Error Handling**: Use hexser error types or standard Result<T, String>.
4. **Async/Await**: All Ports and Adapters must be async.
5. **Testing**: Every new adapter must have an integration test (mocked or using sqlite::memory).

## **Detailed Breakdown for Phase 2 (SQLite-Vec)**

### **Extension Loading**

Ensure the sqlite-vec dynamic library is available or statically linked. In Rust, crates like sqlite-vec might provide build-time bundling.

### **Schema Example**

CREATE VIRTUAL TABLE vec_items USING vec0(  
embedding float[1536]  
);

CREATE TABLE artifacts (  
id INTEGER PRIMARY KEY AUTOINCREMENT,  
uuid TEXT NOT NULL,  
content TEXT,  
-- other fields  
);  
