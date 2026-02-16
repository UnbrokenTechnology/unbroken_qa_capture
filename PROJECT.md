# Project Context

## What does this product do?

Unbroken QA Capture is a lightweight desktop application that streamlines manual software QA. It orchestrates OS-native screen capture via hotkeys (Print Screen starts a bug capture, F4 ends it), provides screenshot annotation with text labels (solving the Windows 11 Snipping Tool text gap), collects application metadata automatically, and uses Claude AI to draft structured bug reports. All evidence is organized into session-based folder structures ready for ticket creation.

The primary interaction model is zero-friction: during active testing the tool stays invisible. At the end of a session, the tester reviews AI-assisted bug descriptions and gets organized output for their ticketing workflow.

## Who are the target users?

Manual QA testers on Windows 11 who find 10-30 bugs per session, have a Claude subscription (Pro/Max), and use Linear for ticketing. They value speed during testing and thoroughness in bug reports.

## Business model / problem solved

QA testers currently spend 5-10 minutes per bug on documentation overhead — organizing screenshots, writing descriptions, attaching evidence to tickets. With 20-30 bugs per session, this overhead is enormous. Unbroken QA Capture reduces this to seconds per bug during capture and minutes total during review, with AI handling the structured write-up.

## Hard constraints

- **v1 platform:** Windows 11 only. macOS is architected (trait-based platform abstraction in Rust) but not implemented.
- **Stack (non-negotiable):** Tauri 2 (Rust backend + WebView2) + Vue 3 + Quasar Framework + TypeScript (Composition API, `<script setup lang="ts">`) + Pinia state management.
- **AI integration:** Claude Code CLI as subprocess (`claude --print`), not direct API. Uses the tester's existing subscription. Stateless invocations only — no accumulated context.
- **No cloud:** All data stays local. No telemetry, no sync.
- **Graceful degradation:** Must function fully as a capture-and-organize tool without Claude CLI installed.
- **Registry safety:** Windows Snipping Tool redirect via HKCU registry must always be restored — even on crash (Rust `Drop` trait + startup recovery check).

## What does success look like?

A tester captures 20-30 bugs in a session using only hotkeys, reviews them with AI-drafted descriptions, and has organized folders with formatted ticket content — all in under 30 minutes including capture time. The tool never steals focus from the application under test.

## Testing Strategy

**Dual-stack testing:**
- **Rust backend:** Built-in `#[cfg(test)]` modules with `cargo test`. Unit tests for session management, file watching logic, metadata collection, Claude CLI subprocess handling, and ticketing formatting. Platform-specific code (Windows registry, Snipping Tool trigger) uses trait-based mocks for testability on non-Windows environments.
- **Vue/TypeScript frontend:** Vitest for component and store unit tests. All Pinia stores and Vue components tested via Composition API patterns. Tauri IPC calls mocked at the invoke boundary.

**Runtime validation (verify.sh):**
- Rust: `cargo clippy` (lint) + `cargo test` (unit tests)
- TypeScript: type checking + Vitest
- Both stacks must pass for verification to succeed

**Integration testing:** End-to-end session lifecycle tests (start -> capture -> end -> review) using mocked OS services. File watcher detection tests. Claude CLI invocation tests with mock subprocess.

**Manual verification:** Windows-specific features (registry redirect, Snipping Tool integration, global hotkeys, tray icon) require manual testing on Windows 11, delegated via verify tickets when needed.
