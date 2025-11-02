use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};

use crate::error::LlmError;

#[derive(Clone)]
pub struct Llm {
    llm: Ollama,
    model: String,
}

const TAGGER_SYSTEM_PROMPT: &str = "You are an expert tagger for an AI art generator. Your task is to extract all relevant tags from the user's prompt. Format the tags as a single, comma-separated list. Use lowercase and snake_case for multi-word tags. Be comprehensive and include tags for subject, appearance, clothing, lighting, background, and overall style.";

impl Llm {
    pub fn new(host: &str, port: u16, model: impl Into<String>) -> Result<Self, LlmError> {
        let ollama = Ollama::new(host, port);

        Ok(Self {
            llm: ollama,
            model: model.into(),
        })
    }

    pub fn ollama(&self) -> &Ollama {
        &self.llm
    }

    pub fn ollama_mut(&mut self) -> &mut Ollama {
        &mut self.llm
    }

    pub async fn generate_candidate_tags(&self, prompt: &str) -> Result<Vec<String>, LlmError> {
        let system_prompt = TAGGER_SYSTEM_PROMPT;

        let full_prompt = format!("USER PROMPT: \"{}\"", prompt);

        let request = GenerationRequest::new(self.model.clone(), full_prompt).system(system_prompt);

        let response = self.llm.generate(request).await?;

        let tags: Vec<String> = response
            .response
            .trim()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(tags)
    }
}
