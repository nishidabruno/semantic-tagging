use ollama_rs::{Ollama, generation::completion::request::GenerationRequest};
use serde::{Deserialize, Serialize};

use crate::error::LlmError;

const TAGGER_SYSTEM_PROMPT: &str = "You are an expert tagger for an AI art generator. Your task is to extract all relevant tags from the user's prompt. Format the tags as a single, comma-separated list. Use lowercase and snake_case for multi-word tags. Be comprehensive and include tags for subject, appearance, clothing, lighting, background, and overall style.";

const STRUCTURED_TAGGER_SYSTEM_PROMPT: &str = r#"
You are an expert AI art tagger. Your task is to analyze the user's prompt and extract all relevant tags, organizing them into a specific JSON structure.

**RULES:**
1.  You MUST respond with a single, valid JSON object. Do not include any explanatory text, comments, or markdown fences like ```json.
2.  The JSON object must have three top-level keys: "subject", "environment", and "quality".
3.  Each key must contain a list of strings (tags).
4.  **Crucially, the tags within each list MUST be ordered from most important to least important.** The most central or visually prominent element should always be first.
5.  All tags must be in lowercase and use snake_case for multi-word phrases.
6.  If a category has no relevant tags, you MUST provide an empty list `[]`.

**EXAMPLE:**
USER PROMPT: "A beautiful portrait of a redhead warrior girl with piercing blue eyes, standing in a dark, enchanted forest at sunset. The lighting is cinematic and the art is highly detailed, 4k."

YOUR RESPONSE:
{
  "subject": [
    "1girl",
    "redhead",
    "warrior",
    "blue_eyes",
    "portrait"
  ],
  "environment": [
    "enchanted_forest",
    "dark",
    "sunset",
    "cinematic_lighting"
  ],
  "quality": [
    "highly_detailed",
    "4k",
    "masterpiece",
    "best_quality"
  ]
}
"#;

#[derive(Debug, Deserialize, Serialize)]
pub struct StructuredTags {
    subject: Vec<String>,
    environment: Vec<String>,
    quality: Vec<String>,
}

impl StructuredTags {
    pub fn to_flat_vec(&self) -> Vec<String> {
        self.subject
            .iter()
            .chain(self.environment.iter())
            .chain(self.quality.iter())
            .cloned() // TODO: is using `cloned` the best here?
            .collect()
    }

    pub fn clean_json_output(raw: String) -> String {
        // trim backticks, quotes, and whitespace
        let trimmed = raw.trim_matches(&['`', '"', '\n', ' '] as &[_]);

        // remove "json" or "json\n"
        let without_prefix = trimmed
            .strip_prefix("json")
            .unwrap_or(trimmed)
            .trim_start_matches("\\n")
            .trim_start();

        without_prefix.to_string()
    }
}

#[derive(Clone)]
pub struct Llm {
    llm: Ollama,
    model: String,
}

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

    pub async fn generate_structured_tags(&self, prompt: &str) -> Result<StructuredTags, LlmError> {
        let request = GenerationRequest::new(self.model.clone(), prompt)
            .system(STRUCTURED_TAGGER_SYSTEM_PROMPT);

        let response = self.ollama().generate(request).await?;

        let clean_response = StructuredTags::clean_json_output(response.response);
        println!("{:?}", clean_response);
        let structured_tags = serde_json::from_str(&clean_response)?;

        Ok(structured_tags)
    }
}
