use std::env;

pub struct Config {
    pub server_addr: String,
    pub qdrant_url: String,
    pub collection_name: String,
    pub ollama_host: String,
    pub ollama_port: u16,
    pub vector_size: u64,
    pub embedding_model: String,
    pub llm_model: String,
    pub embedding_concurrency: usize,
}

impl Config {
    pub fn from_env() -> Self {
        let parse_numeric = |var_name: &str, default_val| {
            env::var(var_name)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default_val)
        };

        let server_addr = env::var("SERVER_ADDR").unwrap_or_else(|_| "0.0.0.0:3333".to_string());

        let qdrant_url =
            env::var("QDRANT_URL").unwrap_or_else(|_| "http://localhost:6334".to_string());

        let collection_name =
            env::var("COLLECTION_NAME").unwrap_or_else(|_| "image-tags".to_string());

        let ollama_host =
            env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost".to_string());

        let llm_model = env::var("LLM_MODEL").unwrap_or_else(|_| "llama3:8b".to_string());

        let embedding_model =
            env::var("EMBEDDING_MODEL").unwrap_or_else(|_| "nomic-embed-text".to_string());

        let ollama_port: u16 = parse_numeric("OLLAMA_PORT", 11434);
        let vector_size: u64 = parse_numeric("VECTOR_SIZE", 768).into();
        let embedding_concurrency: usize = parse_numeric("EMBEDDING_CONCURRENCY", 1).into();

        Self {
            server_addr,
            qdrant_url,
            collection_name,
            ollama_host,
            ollama_port,
            vector_size,
            embedding_model,
            llm_model,
            embedding_concurrency,
        }
    }
}
