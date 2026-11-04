# LearnWeave Implementation Status

> Last updated: 2026-11-04  
> Release candidate: `v1.5.0` on `codex/connector-runtime-unification`

This file is the source of truth for **what is actually implemented now**.

Status meanings:

- **Done** — implemented in the current source tree.
- **Partial** — implemented enough to work, but still has a known architecture, UX, or verification gap.
- **Not done** — intentionally missing.
- **Verification pending** — code exists, but the final real-world or release-level validation is still incomplete.

## Product foundation

| Area | Status | Current state |
| --- | --- | --- |
| Product name | Done | Product and repository use LearnWeave. |
| Tauri application identity | Done | `com.learnweave.app`. |
| Legacy ZhiForge database compatibility | Done | Falls back to the existing ZhiForge data directory when needed. |
| Legacy settings compatibility | Done | Existing ZhiForge config remains readable. |
| Legacy credential compatibility | Done | Existing ZhiForge Windows Credential Manager entries remain readable. |
| Legacy frontend AI localStorage migration | Done | Old `zhiforge.ai-settings.v1` is read and migrated to the LearnWeave key. |
| Repository rename | Done | `Luxciax/LearnWeave`. |

## Capture

| Capability | Status | Current state |
| --- | --- | --- |
| Manual text capture | Done | Global Capture dialog. |
| Public URL capture | Done | Redirect/private-network validation, timeout, response-size and text-size limits. |
| Desktop selection capture | Done | Existing Windows selection workflow retained. |
| Browser Helper capture | Done | Works on ordinary web pages; Zhihu keeps richer metadata. |
| Tray capture entry | Done | Available from tray integration. |
| Global capture shortcut | Done | `Ctrl+Shift+Space`. |
| Text-based PDF import | Done | Local PDF.js extraction; original file is not uploaded. |
| Scanned/image-only PDF OCR | Done | Sparse/image-only pages automatically fall back to local Tesseract OCR; captured text keeps page markers and the original PDF is not uploaded. |

## Inbox and knowledge workflow

| Capability | Status | Current state |
| --- | --- | --- |
| Capture → Inbox | Done | Saved learning material enters Inbox before formal knowledge. |
| Internalization | Done | Builds structured knowledge from captured source text. |
| Evidence/source grounding | Done | Knowledge remains linked to source evidence. |
| Question generation | Done | Recall/explanation/application questions are supported. |
| Assessment | Done | Answers are judged against stored evidence. |
| Review scheduling | Done | Mastery/review workflow retained. |
| Today/task-center recovery flows | Done | Failed/interrupted processing can be surfaced and retried. |
| Research workflow | Done | Research prompts are no longer defined around Zhihu. |

## Source model

| Capability | Status | Current state |
| --- | --- | --- |
| Generic source type | Done | `source_kind` separates web/desktop/document/manual/unknown. |
| Connector identity | Done | `connector_id` records the source integration. |
| Legacy platform compatibility | Done | Existing `platform` remains for compatibility. |
| Existing data backfill | Done | Migration maps legacy source rows to source kind / connector metadata. |
| Source filtering | Done | Source library filters by source kind and connector. |
| Source label normalization | Done | UI prefers connector/source labels rather than old platform-only wording. |

## Discover and connectors

| Connector / area | Status | Current state |
| --- | --- | --- |
| Zhihu | Done | Existing richer search/reader workflow preserved as a connector. |
| Wikipedia | Done | Search implementation exists. |
| arXiv | Done | Search implementation exists. |
| Stack Exchange / Stack Overflow | Done | Search implementation exists. |
| Hacker News | Done | Search implementation exists. |
| RSS / Atom | Done | Feed parsing/fetch path exists. |
| Brave Search | Done | API-key based integration exists. |
| Tavily | Done | API-key based integration exists. |
| Exa | Done | API-key based integration exists. |
| Custom HTTP / JSON | Done | User-configurable GET endpoint and field mappings exist. |
| Connector API keys in Windows Credential Manager | Done | Paid/custom connector secrets are kept out of config files. |
| Rust `SourceConnector` core interface | Done | Core trait, capability metadata, Web URL and Zhihu implementations exist. |
| All Discover searches routed through `SourceConnector` | Done | Wikipedia, arXiv, Stack Exchange, HN, RSS, Brave, Tavily, Exa, Zhihu search and custom HTTP/JSON all cross the shared search boundary. Zhihu keeps its richer reader UI on top of the same connector metadata. |
| Live upstream E2E verification | **Verification pending** | Implementations exist, but every external service has not yet been validated with real live responses/keys. |

## AI routing and usage

| Capability | Status | Current state |
| --- | --- | --- |
| Per-learning-task model routing | Done | Internalize, question generation, assessment, review planning, and research can use separate routes. |
| Fallback model | Done | Task route can define a fallback provider/model. |
| Provider-reported token usage | Done | Input/output/cached token usage is recorded and summarized. |
| OpenAI-compatible | Done | Supported. |
| OpenAI Responses | Done | Supported. |
| Anthropic Messages | Done | Supported. |
| Gemini | Done | Supported. |
| Provider presets | Done | DeepSeek, Qwen/Model Studio, Kimi/Moonshot, GLM, Doubao/Ark, Ollama, LM Studio. |
| Model capability metadata | Done | Capability handling exists in the AI layer. |
| Real-world routing matrix verification | **Verification pending** | Needs testing across actual providers/models and weak/strong model combinations. |

## Search

| Capability | Status | Current state |
| --- | --- | --- |
| Local knowledge search | Done | Local candidate search works. |
| AI semantic reranking | Done | Structured AI reranking exists. |
| Fallback when reranking fails | Done | Falls back to local matches. |
| Rerank reliability/diagnostics | Done | Query-expansion/rerank status and bounded failure diagnostics are returned explicitly; fallback remains non-fatal and visible in the Library UI. |

## Build and verification

The large LearnWeave refactor tree was fully validated in GitHub Actions:

- Workflow run: `41082926349`
- dependency install: passed
- frontend type-check: passed
- Rust tests: passed
- Windows Tauri bundle build: passed
- verification artifact upload: passed

The final follow-up commit `e481074` added the Connector core boundary and legacy frontend AI-storage migration.

For that final delta:

- TypeScript check of the changed frontend migration file: passed
- `cargo check --locked`: passed
- `cargo check --tests --locked`: passed
- a fresh full Windows Tauri bundle CI for this exact final commit: **Verification pending** due to the GitHub Actions runner queue problem encountered during development.

## Remaining work

The code gaps tracked for the generic-learning refactor are complete. Remaining work is verification/release work:

1. Perform live E2E verification for external connectors that require real upstream services/credentials.
2. Verify the AI routing matrix against actual configured providers/models.
3. Run the full Windows release gate against the exact `v1.5.0` release commit.
4. Verify installer/MSI/portable artifacts and published SHA-256 checksums.
5. Optionally rename the local development folder from `ZhiForge` to `LearnWeave`; this is housekeeping only and does not affect the product identity.

## Local-only files

At the time of this update, these are intentionally not part of the tracked project documentation:

- `docs/CODEX_E2E_TEST_REPORT.md`
- `preview/`

Do not treat them as authoritative implementation status unless they are deliberately reviewed and committed later.

## v1.5.0 release-candidate delta

- Unified Discover search execution through `SourceConnector`, including Zhihu search and custom HTTP/JSON profiles.
- Added scanned/image-only PDF OCR fallback with page-aware captured text.
- Added explicit semantic expansion/rerank fallback diagnostics.
- Local `pnpm check` and `pnpm build` pass for this release candidate.
- **Windows/Rust verification pending:** the local MCPX environment does not expose Rust/Cargo, so the strengthened GitHub Actions Windows release gate is the authoritative Rust/test/bundle check for this exact candidate.
