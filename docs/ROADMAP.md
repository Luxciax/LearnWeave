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

## v1.5.0 refactor completion

The generic-learning code goals for this phase are implemented:

- built-in, Zhihu and custom HTTP/JSON Discover searches cross one `SourceConnector` search boundary;
- connector metadata is centralized in the backend while Zhihu keeps its richer reader UX;
- PDF capture extracts embedded text and automatically OCRs sparse/image-only pages locally;
- captured PDF text retains page markers;
- semantic search exposes expansion/rerank fallback state and bounded diagnostics instead of silently hiding degradation;
- existing ZhiForge data/settings/credentials remain readable.

## Current priorities

### 1. Release-quality validation

Before publishing a stable LearnWeave release:

- run frontend type-check and production build;
- run Rust tests against the locked dependency graph;
- run Windows Tauri bundle build;
- verify upgrade from an existing ZhiForge installation;
- verify database/settings/credential fallback and migration;
- verify installer, MSI, portable executable, hashes, and release notes.

The GitHub Actions Windows release workflow is the authoritative release gate for Rust and Windows packaging.

### 2. Complete real-world Connector verification

Run end-to-end checks against live upstream services where credentials/services are available:

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

### 3. Verify AI routing against real providers

Capability metadata and per-task routing are implemented. Remaining work is provider/model verification:

- primary/fallback routing with weak and strong models;
- structured-output behavior;
- timeout/retry diagnostics;
- provider-reported usage accounting.

## Later work

These are useful but are not required for v1.5.0:

- richer document formats beyond PDF;
- optional bundled OCR language packs and OCR quality controls;
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
