# LearnWeave Hackathon Demo Runbook

This runbook uses real persisted product data and the normal LearnWeave flow. Do not replace the learning path with seeded knowledge records.

## Before the demo

- Start LearnWeave normally.
- Confirm an AI provider/model is configured.
- Optionally configure a dedicated learning-task route in **设置 → 学习 AI**.
- If using Zhihu, configure its Access Secret.
- If showing generic browser capture, load browser-extension as an unpacked extension.

## Recommended continuous path

1. Open **发现** and choose a source. Public options include Wikipedia, arXiv, Stack Overflow, Hacker News, RSS/Atom, plus Zhihu when configured.
2. Search or read one real result and choose useful material.
3. Use **加入收件箱**. The Source is persisted before it becomes formal knowledge.
4. Open **收件箱** and wait for extraction/classification.
5. Review drafts and accept selected knowledge.
6. Open a KnowledgeUnit and answer an understanding question without rereading the source.
7. Show source-grounded judgement, Evidence, mastery change, and review state.
8. Reopen the original source when available.
9. Show **今天 / 复习** to demonstrate the continuing learning loop.

## Alternate capture entries

- Ctrl+Shift+Space → pasted text / URL / PDF.
- Tray → 捕获内容.
- Windows selection toolbar → 内化.
- Browser Helper on ordinary HTTP(S) pages; Zhihu uses richer metadata.

Text-based PDFs are parsed locally. Image-only/scanned PDFs need OCR before capture.

## AI routing and usage

Settings → **学习 AI** can route extraction, question generation, assessment, review planning, and research to different primary/fallback models. The same page shows provider-reported token usage.

## Acceptance checklist

- captured material enters Inbox rather than formal knowledge directly;
- accepted knowledge keeps source Evidence;
- question generation and assessment use persisted data;
- per-task AI routing falls back to internalize when not configured;
- generic Discover results can enter Inbox;
- browser/URL captures preserve source metadata;
- PDF text capture uses document/pdf-upload origin;
- restart preserves SQLite data and legacy ZhiForge data remains readable after upgrade.
