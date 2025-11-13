use core::{Config, Embedding, Llm, error::AppError};
use std::sync::Arc;

use axum::{
    Json, Router,
    extract::State,
    response::IntoResponse,
    routing::{get, post},
};
use dotenvy::dotenv;
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct PromptInput {
    prompt: String,
    include_positive: Option<bool>,
    include_negative: Option<bool>,
}

#[derive(Serialize)]
pub struct GenerateTagsResponse {
    #[serde(skip_serializing_if = "Option::is_none")]
    positive: Option<&'static [&'static str]>,
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    negative: Option<&'static [&'static str]>,
}

#[derive(Clone)]
struct AppState {
    embedding: Embedding,
    llm: Llm,
}

const POSITIVE_TAGS: &[&str] = &[
    "masterpiece",
    "best quality",
    "very aesthetic",
    "absurdres",
    "amazing quality",
];

const NEGATIVE_TAGS: &[&str] = &[
    "bad quality",
    "worst quality",
    "worst detail",
    "bad hands",
    "bad anatomy",
    "extra fingers",
];

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();
    let config = Config::from_env();
    tracing_subscriber::fmt::init();

    let llm = Llm::new(config.ollama_host, config.ollama_port, config.llm_model)?;
    let embedding = Embedding::new(
        config.qdrant_url,
        config.vector_size,
        config.embedding_model,
        config.collection_name,
        config.embedding_concurrency,
    )
    .await?;
    let app_state = AppState { embedding, llm };

    let app = Router::new()
        .route("/health", get(check_health))
        .route("/generate-tags", post(generate_tags))
        .with_state(Arc::new(app_state));

    let listener = tokio::net::TcpListener::bind(config.server_addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn check_health() -> &'static str {
    "beep boop"
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

    let positive = input
        .include_positive
        .and_then(|v| v.then_some(POSITIVE_TAGS));
    let negative = input
        .include_negative
        .and_then(|v| v.then_some(NEGATIVE_TAGS));

    Ok(Json(GenerateTagsResponse {
        positive,
        tags,
        negative,
    }))
}
