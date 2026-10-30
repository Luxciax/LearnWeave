# ZhiForge Demo Runbook

This is the short, reliable demonstration path for the hackathon build.

## Before the demo

1. Start ZhiForge and open **Settings**.
2. Confirm one AI provider/model is enabled for the internalization route.
3. Confirm the Zhihu Access Secret is configured if you plan to use live Zhihu search.
4. Keep one known-good Zhihu query ready. If the network or Zhihu API is unavailable, use the desktop selection entry on any prepared article/text instead.

## 3-minute judge flow

### 1. Find useful knowledge

Open **搜知乎**, search a prepared topic, open one result, and choose a useful passage or the full answer.

Click **加入收件箱**.

What to say: ZhiForge does not turn every saved paragraph directly into permanent knowledge. New material first enters an Inbox and keeps the original source as the authority.

### 2. Turn the source into reviewable knowledge

Open **收件箱**. Wait for extraction to finish, inspect the generated drafts, deselect anything unnecessary, then click **接收选中**.

What to say: AI proposes atomic knowledge units and evidence, but the user controls what becomes formal knowledge. Evidence remains linked to the exact source text.

If extraction, evidence validation, or duplicate classification fails, the Inbox shows an explicit retryable failure. It does not silently convert malformed AI output into accepted-looking drafts.

### 3. Show that knowledge becomes learnable

Open the accepted knowledge item and start one understanding question. Before submission, ZhiForge hides the core claim, evidence, personal note, knowledge structure, and original text. Submit an answer, then show the revealed evidence and source link.

What to say: the goal is not another bookmark collection. Knowledge becomes something the user must actively recall first, then verify against the original source.

### 4. Show the daily loop

Open **今天**.

Point out the six compact summaries: Inbox, learning/review, research gaps, knowledge maintenance, organization suggestions, and AI tasks. Then open one concrete item from the action queue.

What to say: Today answers one question: “What should I do next to make this knowledge base more useful?” It is an action queue rather than an analytics dashboard.

### 5. Show knowledge growth

Open **主题扩充**, enter a topic, and run **分析知识缺口**. Pick one gap and search Zhihu. Add one candidate source to the Inbox.

What to say: ZhiForge can compare a learning goal with what the user already knows, identify missing parts, then turn external material into the same controlled learning pipeline.

## Demo fallback and recovery order

If live Zhihu search is unavailable:

1. Use the Windows text-selection toolbar on prepared text.
2. Capture it into ZhiForge.
3. Continue from **收件箱** → **知识库** → **今天**.

If an AI response is malformed, the item enters an explicit failed state. Fix the Provider/model issue if necessary and use the retry path; do not present degraded deterministic output as successful AI internalization.

If an AI task fails or remains stuck for too long:

1. Open **今天** → **AI 任务**.
2. The task center converts stale processing jobs into explicit retryable failures instead of leaving an endless spinner.
3. Click **重试**. If the expected drafts or questions were already committed before the failure, ZhiForge reconciles the task to completed instead of duplicating work.
4. For extraction failures, **收件箱** shows the same failed state and retry path. A question-generation failure does not move already accepted knowledge back to a failed Inbox state.

## Do not depend on during the short demo

The graph, advanced curation, backup/restore, deep review scheduling, and semantic search are useful supporting features, but they are not required to prove the primary product story. Keep the judge flow focused on:

**Source → Inbox → Review/Accept → Focused Recall → Evidence → Today → Topic Research**
