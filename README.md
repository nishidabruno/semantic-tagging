# WIP 🚧

Translates natural language prompts into optimized, structured tags for image generation. Uses LLM for semantic extraction and a vector database for tag canonicalization via semantic search.

For example, it turns this:
> "A beautiful portrait of a redhead warrior girl with piercing blue eyes, standing in a dark, enchanted forest at sunset."

Into this:
> `1girl, red_hair, warrior, blue_eyes, portrait, enchanted_forest, dark, sunset, cinematic_lighting, highly_detailed, masterpiece`
<img width="1744" height="972" alt="example" src="https://github.com/user-attachments/assets/83078d81-dab6-4f08-9bb1-53a3e3336696" />

## Core Features

-   **Tag Correction & Optimization:** Performs cosine similarity searches on the vector database using tag embeddings to identify the closest semantic matches, replacing them with standardized or community-preferred tags (e.g., it knows to convert `girl` to `1girl`).
-   **Structured & Prioritized Tagging:** Automatically organizes tags into logical categories (**Subject**, **Environment**, **Quality**) and orders them from most to least important.
-   **High-Performance API:** Built to be fast and reliable.

## How It Works

The service provides a simple API that triggers a powerful, three-step pipeline:

1.  **Generation:** First, it analyzes the user's prompt to generate a broad list of "candidate" tags, sorting them into categories like the main subject, the background, and the overall style.
2.  **Validation:** Next, it takes each candidate tag and searches a vast knowledge base of expert-approved tags to find the closest and most effective match. This is where "pink hair" becomes the more standard `pink_hair`.
3.  **Finalization:** Finally, it compiles a clean, de-duplicated list of the best tags, correctly prioritized, and returns it to the user.

## Getting Started

### Prerequisites

You will need **Ollama** and **Qdrant** running. The easiest way to get started is with Docker.

1.  **Install Ollama:** Follow the official instructions at [ollama.com](https://ollama.com/). Then, download the necessary AI models:
    ```bash
    ollama pull gemma3:4b
    ollama pull embeddinggemma
    ```
2.  **Run Qdrant (Knowledge Base):**
    ```bash
    docker run -p 6333:6333 -p 6334:6334 \
        -v $(pwd)/qdrant_storage:/qdrant/storage:z \
        qdrant/qdrant
    ```

## API Usage

### Generate Optimized Tags

This is the main endpoint for transforming a prompt into a validated tag set.

-   **Endpoint:** `POST /generate-tags`
-   **Request Body:**
    ```json
    {
      "prompt": "A beautiful portrait of a redhead warrior girl with piercing blue eyes, standing in a dark, enchanted forest at sunset."
    }
    ```
-   **Success Response (200 OK):**
    ```json
    [
      "1girl",
      "red_hair",
      "warrior",
      "blue_eyes",
      "portrait",
      "enchanted_forest",
      "dark",
      "sunset",
      "cinematic_lighting",
      "highly_detailed",
      "masterpiece"
    ]
    ```

---

## Future Roadmap

Some of the planned improvements:

-   **[ ] Automatic Negative Prompts:** Automatically generate tags for things to *avoid* in the image (e.g., `bad_anatomy`, `blurry`).
-   **[ ] Tag Weighting:** Allow the system to identify the most critical tags and automatically add emphasis to them (e.g., `(1girl:1.2)`).
-   **[ ] Caching:** Implement a caching system to provide instant results for frequently used prompts.
-   **[ ] Batch Processing:** Add an endpoint to process multiple prompts in a single request.
