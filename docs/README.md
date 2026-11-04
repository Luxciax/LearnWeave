# LearnWeave Documentation

LearnWeave is the current product name; **ZhiForge** is the former name retained only where compatibility or history requires it.

## Start here

- [../README.md](../README.md) — product overview, current user-facing capabilities, setup and compatibility notes.
- [ROADMAP.md](ROADMAP.md) — current product direction and what should be built next.
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) — current implementation progress, known gaps, verification state and remaining work.
- [VERSIONING.md](VERSIONING.md) — release/version policy and release gate.

For day-to-day development, **IMPLEMENTATION_STATUS.md is the progress source of truth**. Update it whenever a feature changes status or verification state.

## Current product principles

1. Capture is separate from Discover.
2. Saved learning material enters Inbox before formal knowledge.
3. Knowledge remains linked to source Evidence.
4. AI task routing is independent from one-shot desktop actions.
5. Source type is stable; platform-specific behavior belongs to Connectors.
6. Secrets remain outside configuration files.
7. Existing ZhiForge data and credentials remain readable during the rename transition.

## Testing and demo documents

- [CODEX_E2E_TEST_PLAN.md](CODEX_E2E_TEST_PLAN.md) — E2E test plan.
- [HACKATHON_DEMO.md](HACKATHON_DEMO.md) — demo flow/reference.

These describe testing or demo workflows; they are not the implementation-progress source of truth.

## Historical design documents

The following documents describe the earlier SelectionTranslator/ZhiForge-era architecture and are retained for reference only:

- [v0.2-product-spec.md](v0.2-product-spec.md)
- [v0.2-core-architecture.md](v0.2-core-architecture.md)
- [v0.2-extension-maintenance-guide.md](v0.2-extension-maintenance-guide.md)

They do **not** override the current LearnWeave product flow, naming, storage paths, Connector model, roadmap, or release process.

## Documentation maintenance rule

When an Agent completes development work that changes product scope:

1. update `IMPLEMENTATION_STATUS.md`;
2. update `ROADMAP.md` if priorities or scope changed;
3. update the root `README.md` only when user-facing capabilities or setup changed;
4. do not rewrite historical `v0.2-*` documents to look current.
