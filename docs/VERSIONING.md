# Versioning and Release Policy

LearnWeave uses Semantic Versioning for public releases.

## Version synchronization

Before every release, package.json, src-tauri/Cargo.toml, the learnweave package entry in Cargo.lock, and src-tauri/tauri.conf.json must match.

## Release artifacts

A stable Windows release should include the Tauri installer bundle(s), LearnWeave-vX.Y.Z-Windows-x64-Portable.exe, SHA-256 checksums, and release notes.

## Release gate

1. pnpm check
2. pnpm build
3. cargo test --manifest-path src-tauri/Cargo.toml
4. cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings when used for a stable release gate
5. pnpm tauri build
6. verify installer/MSI/portable artifact names and hashes
7. commit and tag vX.Y.Z

Versions before 1.0.0 were internal development snapshots.
