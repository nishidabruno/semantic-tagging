use axum::{
    Json,
    response::{IntoResponse, Response},
};
use ollama_rs::error::OllamaError;
use qdrant_client::QdrantError;
use reqwest::StatusCode;
use serde_json::json;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum LlmError {
    #[error("Failed to generate response from LLM")]
    Generation(#[from] OllamaError),
}

#[derive(Debug, Error)]
pub enum EmbeddingError {
    #[error("Failed build Qdrant client")]
    ClientBuild(#[from] QdrantError),

    #[error("Ollama interaction failed: {0}")]
    Llm(#[from] LlmError),

    #[error("Vector database operation failed: {0}")]
    VectorDatabase(String),

    #[error("Could not find a generated embedding in the response")]
    EmbeddingNotFound,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("LLM interaction failed: {0}")]
    Llm(#[from] LlmError),

    #[error("Embedding or vector DB operation failed: {0}")]
    Embedding(#[from] EmbeddingError),

    // #[error("Failed to read CSV file: {0}")]
    // Csv(#[from] CsvError),
    #[error("Failed to initialize server: {0}")]
    ServerBind(#[from] std::io::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AppError::Llm(_) => (StatusCode::INTERNAL_SERVER_ERROR, "LLM operation failed"),
            AppError::Embedding(e) => {
                eprintln!("Embedding Error: {:?}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Database or embedding operation failed",
                )
            }
            // AppError::Csv(_) => (
            //     StatusCode::INTERNAL_SERVER_ERROR,
            //     "Failed to process data file",
            // ),
            AppError::ServerBind(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "Server failed to start")
            }
        };

        let body = Json(json!({ "error": error_message }));

        (status, body).into_response()
    }
}
