use std::collections::HashSet;

use futures::StreamExt;
use futures::TryStreamExt;
use futures::stream;
use ollama_rs::generation::embeddings::request::GenerateEmbeddingsRequest;
use qdrant_client::QdrantError;
use qdrant_client::{
    Qdrant,
    qdrant::{
        CreateCollectionBuilder, Distance, PointStruct, SearchPoints, UpsertPointsBuilder,
        VectorParamsBuilder,
    },
};

use crate::error::EmbeddingError;
use crate::error::LlmError;
use crate::{TagOutput, csv::TagRow, llm::Llm};

const COLLECTION_EXISTS_CODE: i32 = 6;

#[derive(Clone)]
pub struct Embedding {
    db: Qdrant,
    collection_name: String,
    embedding_model: String,
    embedding_concurrency: usize,
}

impl Embedding {
    pub async fn new(
        url: impl Into<String>,
        size: u64,
        embedding_model: impl Into<String>,
        collection_name: impl Into<String>,
        embedding_concurrency: usize,
    ) -> Result<Self, EmbeddingError> {
        let collection_name = collection_name.into();
        let embedding_model = embedding_model.into();

        let db = qdrant_client::Qdrant::from_url(&url.into()).build()?;

        let create_response = db
            .create_collection(
                CreateCollectionBuilder::new(&collection_name)
                    .vectors_config(VectorParamsBuilder::new(size, Distance::Cosine)),
            )
            .await;

        match create_response {
            Ok(_) => {
                println!("Collection '{}' created successfully.", collection_name);
            }

            Err(QdrantError::ResponseError { status })
                if status.code() == COLLECTION_EXISTS_CODE.into() =>
            {
                println!("Using collection '{}'", collection_name);
            }

            Err(e) => {
                return Err(e.into());
            }
        }

        Ok(Self {
            db,
            collection_name,
            embedding_model,
            embedding_concurrency,
        })
    }

    async fn generate_single_embedding(
        &self,
        text: &str,
        llm: &Llm,
    ) -> Result<Vec<f32>, EmbeddingError> {
        let request = GenerateEmbeddingsRequest::new(self.embedding_model.clone(), text.into());
        let response = llm
            .ollama()
            .generate_embeddings(request)
            .await
            .map_err(|e| EmbeddingError::Llm(LlmError::Generation(e)))?;

        let embedding = response
            .embeddings
            .into_iter()
            .next()
            .ok_or(EmbeddingError::EmbeddingNotFound)?;

        Ok(embedding)
    }

    pub async fn upsert_batch(&self, rows: Vec<TagRow>, llm: &Llm) -> Result<(), EmbeddingError> {
        println!("Embedding {} rows concurrently...", rows.len());

        let point_futures = stream::iter(rows)
            .map(|record| async move {
                let embedding_vector = self.generate_single_embedding(&record.name, llm).await?;

                Ok::<PointStruct, EmbeddingError>(PointStruct::new(
                    record.tag_id,
                    embedding_vector,
                    [
                        ("name", record.name.as_str().into()),
                        ("category", (record.category as i64).into()),
                    ],
                ))
            })
            .buffer_unordered(self.embedding_concurrency);

        let points: Vec<PointStruct> = point_futures.try_collect().await?;

        println!(
            "Generated {} embeddings. Upserting to database...",
            points.len()
        );

        let upsert_builder = UpsertPointsBuilder::new(&self.collection_name, points);

        self.db.upsert_points(upsert_builder).await?;

        println!("Embedding finished successfully.");
        Ok(())
    }

    pub async fn search(&self, prompt: &str, llm: &Llm) -> Result<Vec<TagOutput>, EmbeddingError> {
        let prompt_embedding = self.generate_single_embedding(&prompt, llm).await?;

        let search_request = SearchPoints {
            collection_name: self.collection_name.clone(),
            vector: prompt_embedding,
            limit: 32,
            with_payload: Some(true.into()),
            score_threshold: Some(0.6),
            ..Default::default()
        };

        let search_result = self.db.search_points(search_request).await?;

        let outputs = search_result
            .result
            .into_iter()
            .filter_map(|point| {
                let name = point.payload.get("name")?.as_str()?.to_string();
                Some(TagOutput {
                    name,
                    score: point.score,
                })
            })
            .collect();

        Ok(outputs)
    }

    /// From a list of candidate tags, performs a semantic search for each one and
    /// retrieves the best match from the vector database.
    ///
    /// ### Example:
    /// ```
    /// Input: ["girl", "hair pink"]
    /// Output: ["1girl", "pink_hair"]
    /// ```
    pub async fn validate_tags_concurrently(
        &self,
        candidate_tags: Vec<String>,
        llm: &Llm,
    ) -> Result<HashSet<String>, EmbeddingError> {
        let validated_tags_stream = stream::iter(candidate_tags)
            .map(|tag_name| async move {
                let embedding_vector = self.generate_single_embedding(&tag_name, llm).await?;

                let search_request = SearchPoints {
                    collection_name: self.collection_name.clone(),
                    vector: embedding_vector,
                    limit: 1,
                    with_payload: Some(true.into()),
                    score_threshold: Some(0.8),
                    ..Default::default()
                };

                let search_result = self.db.search_points(search_request).await?;

                if let Some(top_point) = search_result.result.into_iter().next()
                    && let Some(canonical_name) =
                        top_point.payload.get("name").and_then(|v| v.as_str())
                {
                    println!(
                        "    - Candidate '{}' validated as -> '{}' (Score: {:.2})",
                        tag_name, canonical_name, top_point.score
                    );
                    return Ok(Some(canonical_name.to_string()));
                }

                println!(
                    "    - Candidate '{}' did not find a confident match in the vector database.",
                    tag_name
                );
                Ok(None)
            })
            .buffer_unordered(self.embedding_concurrency);

        let final_tags: HashSet<String> = validated_tags_stream
            .filter_map(
                |res: Result<Option<String>, EmbeddingError>| async move { res.ok().flatten() },
            )
            .collect()
            .await;

        Ok(final_tags)
    }
}
