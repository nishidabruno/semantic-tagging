use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

mod csv;
mod embedding;
mod error;
mod llm;

use csv::read_tags_from_csv;
use embedding::Embedding;

use crate::{error::AppError, llm::Llm};

#[derive(Deserialize)]
pub struct PromptInput {
    prompt: String,
}

#[derive(Debug, Serialize)]
pub struct TagOutput {
    pub name: String,
    pub score: f32,
}

// Qdrant
const QDRANT_URL: &str = "http://localhost:6334";
const COLLECTION_NAME: &str = "image-tags";
// Ollama
const OLLAMA_HOST: &str = "http://localhost";
const OLLAMA_PORT: u16 = 11434;

// embeddinggemma has a dimension of 768.
const VECTOR_SIZE: u64 = 768;
const EMBEDDING_MODEL: &str = "embeddinggemma";

#[derive(Clone)]
struct AppState {
    embedding: Embedding,
    llm: Llm,
}

#[tokio::main]
async fn main() -> Result<(), AppError> {
    tracing_subscriber::fmt::init();

    let llm = Llm::new(OLLAMA_HOST, OLLAMA_PORT, "gemma3:4b")?;
    let embedding =
        Embedding::new(QDRANT_URL, VECTOR_SIZE, EMBEDDING_MODEL, COLLECTION_NAME).await?;
    let app_state = AppState { embedding, llm };

    let app = Router::new()
        .route("/health", get(check_health))
        .route("/tags", post(embed_tags_from_csv))
        .route("/get-tags", post(prompt_to_tags))
        .route("/generate-candidate-tags", post(generate_candidate_tags))
        .route("/generate-tags", post(generate_tags))
        .route("/structured-tags", post(test_generate_structured_tags))
        .with_state(Arc::new(app_state));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3333").await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_health() -> &'static str {
    "bip bip"
}

async fn embed_tags_from_csv(
    State(state): State<Arc<AppState>>,
    // Json(input): Json<PromptInput>,
) -> Result<impl IntoResponse, AppError> {
    let tags = read_tags_from_csv("./selected_tags.csv");

    state.embedding.upsert_batch(tags, &state.llm).await?;

    Ok(())
}

async fn prompt_to_tags(
    State(state): State<Arc<AppState>>,
    Json(input): Json<PromptInput>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.embedding.search(input.prompt, &state.llm).await?;

    Ok(Json(result))
}

async fn generate_candidate_tags(
    State(state): State<Arc<AppState>>,
    Json(input): Json<PromptInput>,
) -> Result<impl IntoResponse, AppError> {
    let result = state.llm.generate_candidate_tags(&input.prompt).await?;

    Ok(Json(result))
}

async fn generate_tags(
    State(state): State<Arc<AppState>>,
    Json(input): Json<PromptInput>,
) -> Result<impl IntoResponse, AppError> {
    let structured_tags = state.llm.generate_structured_tags(&input.prompt).await?;
    let flat_tag_vec = structured_tags.to_flat_vec();

    let tags = state
        .embedding
        .validate_tags_concurrently(flat_tag_vec, &state.llm)
        .await?;

    Ok(Json(tags))
}

async fn test_generate_structured_tags(
    State(state): State<Arc<AppState>>,
    Json(input): Json<PromptInput>,
) -> Result<impl IntoResponse, AppError> {
    let structured_tags = state.llm.generate_structured_tags(&input.prompt).await?;

    Ok(Json(structured_tags))
}
