# WIP 🚧

Translates natural language prompts into optimized, structured tags for image generation. Uses LLM for candidate extraction and a vector database for semantic search.

For example, it turns this:
> "Close-up colorful songbirds on branch, eagle soaring above, soft sunrise light, misty forest background, cinematic focus, ultra-realistic detail."

Into this:

> `multiple_songbird, animal_focus, close-up, branch, tree_branch, forest, mist, sunrise, sky, flying, soaring, wildlife, nature, depth_of_field, cinematic_lighting, realistic, ultra_detailed`
<img width="1744" height="972" alt="Frame 22" src="https://github.com/user-attachments/assets/ed603ffa-41a6-4a0a-83e4-e382ff1fd2f2" />

## Core Features

-   **Tag Correction & Optimization:** Performs cosine similarity searches on the vector database using tag embeddings to identify the closest semantic matches, replacing them with standardized or community-preferred tags (e.g., it knows to convert `girl` to `1girl`).
-   **Structured & Prioritized Tagging:** Automatically organizes tags into logical categories (**Subject**, **Environment**, **Quality**) and orders them from most to least important.
-   **High-Performance API:** Built to be fast and reliable.

## How it works

The service provides a simple API with three-step pipeline:

1.  **Generation:** First, it analyzes the user's prompt to generate a list of "candidate" tags, sorting them into categories like the main subject, the background, and the overall style.
2.  **Validation:** Next, it takes each candidate tag and searches on the vector database to find the closest semantic match. This is where "pink hair" becomes the more standard `pink_hair`.
3.  **Finalization:** Finally, de-duplicated list of the best tags, correctly prioritized, is returned to the user.

## Getting Started

### Prerequisites

To run it locally you will need **Ollama** and **Qdrant**.

1.  **Install Ollama:** Follow the official instructions at [ollama.com](https://ollama.com/). Then, download the necessary models:
    ```bash
    ollama pull gemma3:4b
    ollama pull embeddinggemma
    ```
2.  **Run Qdrant (vector database):** The easiest way to get started is with Docker:
    ```bash
    docker run -p 6333:6333 -p 6334:6334 \
        -v $(pwd)/qdrant_storage:/qdrant/storage:z \
        qdrant/qdrant
    ```

## API Usage

### Generate Tags

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

## Roadmap

Some of the planned improvements:

-   **[ ] Update tags list:** Add more tags to the database to improve tagging accuracy (currently dataset is ~10k).
-   **[ ] Allow external API processing:** Allow using OpenAI models for faster embedding processing and use GPT models.
-   **[ ] Automatic Negative Prompts:** Automatically generate tags for things to *avoid* in the image (e.g., `bad_anatomy`, `blurry`).
-   **[ ] Tag Weighting:** Allow the system to identify the most critical tags and automatically add emphasis to them (e.g., `(1girl:1.2)`).
-   **[ ] Batch Processing:** Add an endpoint to process multiple prompts in a single request.
