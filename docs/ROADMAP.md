# LearnWeave Roadmap

> Last updated: 2026-11-04

This document defines the **current product direction**. Historical `v0.2-*` documents are retained only as design references and do not override this roadmap.

## Product direction

LearnWeave is a local-first Windows learning workstation.

The core learning loop is:

**Capture → Inbox → Knowledge → Active Recall → Feedback → Review**

External discovery is a separate path:

**Discover → choose useful material → Capture → Inbox**

One-shot desktop AI actions such as translation, explanation, summarization, polishing, and custom prompts remain separate from the learning Inbox.

## Current priorities

### 1. Finish the generic Connector runtime

The Connector model already exists, but the runtime is not fully unified yet.

Target:

- every built-in source exposes stable connector metadata;
- search/fetch behavior goes through one Connector boundary;
- source conversion always produces the same generic source model;
- Zhihu remains a richer connector, not a special product path;
- custom HTTP/JSON connectors use the same execution boundary where practical.

Success condition: adding a new source should not require editing the main Discover dispatcher.

### 2. Complete real-world Connector verification

Run end-to-end tests against live upstream services:

- Wikipedia
- arXiv
- Stack Exchange
- Hacker News
- RSS / Atom
- Zhihu
- Brave Search
- Tavily
- Exa
- custom HTTP / JSON connectors

Verify authentication, field mapping, rate limits, empty results, malformed responses, and capture-to-Inbox behavior.

### 3. Improve PDF ingestion

Current PDF support extracts embedded text locally.

Next:

- add OCR for scanned/image-only PDFs;
- preserve page metadata where useful;
- keep original PDF local;
- avoid silently treating an empty scanned PDF as successful text capture.

### 4. Strengthen semantic search and model capability handling

Semantic reranking already exists with local-search fallback.

Next:

- make capability detection explicit per model/provider;
- make rerank fallback state visible without presenting it as a hard failure;
- improve retry/error diagnostics;
- verify task routing with weak/strong model combinations.

### 5. Release-quality validation

Before a stable LearnWeave release:

- run frontend type-check/build;
- run full Rust tests;
- run Windows Tauri bundle build;
- verify upgrade from an existing ZhiForge installation;
- verify database/settings/credential fallback and migration;
- verify installer, MSI, portable executable, hashes, and release notes.

See `VERSIONING.md` for the release gate.

## Later work

These are useful but are not required to complete the current generic-learning refactor:

- richer document formats beyond PDF;
- optional OCR language packs and OCR quality controls;
- connector health/status diagnostics;
- connector-specific ranking controls;
- richer source metadata and provenance display;
- improved knowledge graph exploration;
- stronger offline/local-model workflows;
- import/export improvements for larger personal knowledge bases.

## Non-goals for the current phase

Do not turn LearnWeave into a heavy plugin platform yet.

Avoid adding:

- plugin lifecycle management;
- OAuth framework abstractions before a connector actually needs them;
- event-bus architecture for simple source integrations;
- large framework migrations unrelated to the learning workflow.

Prefer small, static, inspectable connectors and keep the core workflow understandable.
