use core::{Config, Embedding, Llm, csv::read_tags_from_csv, error::AppError};
use std::{env, path::Path};

use dotenvy::dotenv;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    dotenv().ok();
    let config = Config::from_env();

    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "Usage: ./{} <csv_file_path/file.csv>",
            args.first().unwrap()
        );
        std::process::exit(1);
    }

    let path = Path::new(&args[1]);
    let input_tags = read_tags_from_csv(path);

    let llm = Llm::new(config.ollama_host, config.ollama_port, config.llm_model)?;
    let embedding = Embedding::new(
        config.qdrant_url,
        config.vector_size,
        config.embedding_model,
        config.collection_name,
        config.embedding_concurrency,
    )
    .await?;

    embedding.upsert_batch(input_tags, &llm).await?;

    Ok(())
}
