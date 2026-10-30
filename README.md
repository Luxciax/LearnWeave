# LearnWeave

LearnWeave is a local-first Windows knowledge learning workstation for turning useful information into knowledge you can recall, explain, and reuse.

The core loop is **Capture → Inbox → Knowledge → Active Recall → Feedback → Review**. One-shot desktop AI actions such as translation, explanation, summarization, polishing, and custom prompts remain separate from the knowledge Inbox.

## What LearnWeave Does

- Capture selected desktop text, pasted text, public web URLs, and text-based PDFs.
- Use the optional Browser Helper to enrich selections from ordinary web pages; Zhihu keeps richer page-specific metadata.
- Search external sources from **Discover**: Wikipedia, arXiv, Stack Overflow, Hacker News, RSS/Atom, Zhihu, Brave Search, Tavily, Exa, and user-defined HTTP/JSON connectors.
- Route every saved learning source through the Inbox before it becomes formal knowledge.
- Extract source-grounded knowledge units and Evidence.
- Generate recall, explanation, and application questions.
- Judge answers against stored source Evidence rather than outside facts.
- Track mastery and schedule weak or due knowledge for review.
- Configure different AI models for extraction, question generation, assessment, review planning, and research.
- Record provider-reported token usage by learning task.
- Keep provider and connector credentials in Windows Credential Manager.

## Capture

The global **捕获** entry supports pasted text, public HTTP(S) URLs, local PDF text extraction, desktop selection, Browser Helper context, and Zhihu Reader selection.

You can open Capture from the sidebar, tray menu, or the global **Ctrl+Shift+Space** shortcut.

URL capture blocks localhost/private-network targets, validates redirect destinations, applies a 20-second timeout, and limits responses to 5 MB.

PDF capture is local-only and currently extracts embedded text. Scanned/image-only PDFs need OCR before capture.

## Discover and Connectors

Discover is for finding material you do not already have. Capture is for material you already have.

Built-in public connectors use anonymous access where the upstream API permits it. Brave, Tavily, and Exa use user-supplied keys stored in Windows Credential Manager. Custom HTTP/JSON connectors support a GET search endpoint, query parameter, dot-path mappings for result fields, and an optional API-key header.

Zhihu remains a dedicated connector with its existing search/reader workflow instead of defining the product itself.

## AI Routing

LearnWeave supports OpenAI-compatible Chat Completions, OpenAI Responses, Anthropic Messages, and Google Gemini.

Learning tasks can use independent primary/fallback models. If a task has no dedicated route it falls back to the existing internalize route. Structured learning tasks use native JSON response mode where the protocol supports it, with parse/validation and one repair pass still enforcing the final contract.

Provider presets are included for DeepSeek, Qwen/Model Studio, Kimi/Moonshot, GLM, Doubao/Ark, Ollama, and LM Studio. Presets fill protocol and API base only; model names remain user-selectable.

## Local Data and Upgrade Compatibility

Knowledge data is stored locally in SQLite. Settings and secrets stay local.

The application identity is now **LearnWeave** (com.learnweave.app). Existing ZhiForge installations remain readable:

- if the new LearnWeave database does not exist, LearnWeave can continue using the existing ZhiForge database directory;
- settings fall back to the existing ZhiForge config when no new LearnWeave config exists;
- AI and Zhihu credentials fall back to the legacy ZhiForge Windows Credential Manager service.

The compatibility identifiers are intentionally retained in code so upgrading does not make existing data or credentials appear lost.

## Downloads

Windows builds are published through GitHub Releases:

https://github.com/Luxciax/LearnWeave/releases/latest

The Windows workflow builds Tauri NSIS/MSI bundles and a portable executable when available.

## Development

Requirements: Windows, Node.js 22+, pnpm, Rust stable with the MSVC toolchain, and the Tauri 2 Windows prerequisites.

Standard validation: pnpm check, pnpm build, cargo test --manifest-path src-tauri/Cargo.toml, and pnpm tauri build.

## Technology

- Tauri 2 / Rust
- React 19 / TypeScript / Vite
- SQLite via rusqlite
- PDF.js for local PDF text extraction
- Windows UI Automation
- Windows Credential Manager

## Attribution

LearnWeave, formerly ZhiForge, is an independent project. Its early desktop selection interaction research was informed by Cherry Studio, including selection-translation patterns and model-provider compatibility concepts.

Cherry Studio: https://github.com/CherryHQ/cherry-studio

LearnWeave is not affiliated with, endorsed by, or an official fork of Cherry Studio or CherryHQ. See NOTICE.

## License

Starting with **v1.4.0**, the project is source-available under the **PolyForm Noncommercial License 1.0.0**. Commercial or business use requires a separate written commercial license.

See LICENSE, COMMERCIAL_LICENSE.md, and NOTICE. Versions released through **v1.3.0** were distributed under AGPL-3.0-only; those prior grants are not retroactively revoked.
