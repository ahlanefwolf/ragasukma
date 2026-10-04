# Plan: AI-Generated Conversation Summary & Long-Term Memory

## 1. Overview
Implement an automated background summarization mechanism for conversation sessions. Instead of reloading full historical raw chat logs (`assets/histories/*.md`) which wastes tokens and context window, the AI will periodically generate concise factual summaries.

## 2. Key Objectives
- **Filter Out Small Talk**: Exclude casual greetings, polite fillers, and transient small talk.
- **Extract Essential Context**:
  - User preferences, habits, and explicit requests.
  - Active/pending tasks, goals, or unresolved instructions.
  - Key technical decisions and parameters discussed.
- **Token Efficiency**: Compact the context into 3–5 bullet points or structured JSON/Markdown, preserving long-term memory at a fraction of token costs.

## 3. Architecture & Flow

```
[ Active Session / Raw Chat ]
             │
             ▼ (End of session / Periodic trigger)
    [ AI Background Summarizer ]
    Prompt: "Extract only core facts, user preferences, and decisions. Ignore greetings & small talk."
             │
             ▼
    [ Memory Storage: assets/memories/ or assets/histories/summary/ ]
             │
             ▼ (When needed / Relevant query)
    [ Injected as Long-Term Context into System Prompt ]
```

## 4. Proposed Implementation Steps
1. **Trigger Condition**:
   - On session reset / idle timeout / shutdown, or when active turns reach a threshold (e.g., 20+ turns).
2. **Summarizer Worker**:
   - An asynchronous call to LLM using a dedicated, concise prompt template.
   - Structured output containing:
     - `key_facts`: List of factual data learned about the user / workspace.
     - `user_preferences`: Formatting or behavioral preferences.
     - `pending_tasks`: Tasks left unfinished or intended for follow-up.
3. **Storage**:
   - Save summary alongside daily histories or in a persistent memory file (`assets/memory/user_context.md`).
4. **Context Injection**:
   - Only inject relevant summary memory into the LLM system prompt when starting a new session or answering memory-related questions.
