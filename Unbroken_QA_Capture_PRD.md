# Unbroken QA Capture — Product Requirements Document

**Version:** 1.0  
**Date:** February 13, 2026  
**Status:** Draft for Swarm System Intake  
**Stack:** Tauri 2 + Vue 3 + Quasar + TypeScript  
**Company:** Unbroken Technology  

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Problem Statement](#2-problem-statement)
3. [Product Overview](#3-product-overview)
4. [User Personas](#4-user-personas)
5. [User Flow & Interaction Model](#5-user-flow--interaction-model)
6. [Application States & State Machine](#6-application-states--state-machine)
7. [Architecture](#7-architecture)
8. [Module Specifications](#8-module-specifications)
9. [Data Model](#9-data-model)
10. [Folder Structure & Output Format](#10-folder-structure--output-format)
11. [Claude CLI Integration](#11-claude-cli-integration)
12. [Ticketing Integration (Pluggable)](#12-ticketing-integration-pluggable)
13. [First-Run Setup & Settings](#13-first-run-setup--settings)
14. [UI/UX Specifications](#14-uiux-specifications)
15. [Performance & Quality Requirements](#15-performance--quality-requirements)
16. [Security Considerations](#16-security-considerations)
17. [Platform Strategy](#17-platform-strategy)
18. [Build Prioritization & Dependency Graph](#18-build-prioritization--dependency-graph)
19. [Deliverables](#19-deliverables)
20. [Glossary](#20-glossary)

---

## 1. Executive Summary

Unbroken QA Capture is a lightweight desktop application that streamlines the manual software QA workflow. It orchestrates OS-native screen capture, provides annotation capabilities (solving the Windows 11 text annotation gap), collects application metadata, uses Claude AI to draft structured bug reports, and organizes all output into session-based folder structures that feed into existing ticketing workflows.

The tool is designed to stay out of the way during active testing. The primary interaction model is: **Print Screen starts a bug capture, F4 ends it.** Between those two keystrokes, everything the tester captures — screenshots, video, pasted URLs, quick notes — is automatically associated with that bug. At the end of a session, the tester reviews each bug with Claude AI assistance and gets organized folders ready for ticket creation.

**Key differentiator:** Unlike simple screenshot tools, Unbroken QA Capture uses Claude AI (via Claude Code CLI, using the tester's existing subscription) to parse console screenshots, draft bug descriptions from visual evidence, and allow interactive refinement of ticket content — all with carefully scoped, small-context AI invocations that keep the tool fast and cost-effective.

**Target stack:** Tauri 2 (Rust backend + WebView frontend), Vue 3 + TypeScript + Quasar Framework (Composition API), Pinia state management, Claude Code CLI for AI features.

**v1 platform:** Windows 11. macOS is architected in but not implemented in v1.

---

## 2. Problem Statement

### Current Workflow (Manual)

When a QA tester finds a bug today, they must:

1. Trigger a screenshot or screen recording using the OS tool (Windows Snipping Tool, macOS Screenshot).
2. Save the capture somewhere they can find it later — often the default Screenshots folder, mixed in with everything else.
3. If they need to annotate the screenshot on Windows 11, they are stuck. The Snipping Tool supports pen drawing but **cannot add text labels**. They must open a separate tool (Paint, Paint 3D, or a third-party editor), add text, and re-save.
4. Separately note the software version of the application under test.
5. Separately capture context like the meeting ID or meeting link (in the case of meeting software, this requires pressing Ctrl+K in the app, copying the link, and pasting it somewhere).
6. If they need console output, they must screenshot the console separately and remember which console screenshot goes with which bug.
7. Open Linear (or another ticketing system) and manually create a ticket.
8. Write a description from memory and scattered notes.
9. Manually attach all the screenshots and videos to the ticket.
10. Repeat steps 1-9 for every bug found in the session.

This process takes 5-10 minutes per bug. In a session where 20-30 bugs are found, the overhead is enormous. Worse, the context-switching between "testing mode" and "documentation mode" means testers either slow down to document properly or rush through documentation and produce incomplete tickets.

### Key Pain Points

- **Windows 11 annotation gap:** Snipping Tool cannot add text labels to screenshots. This is the single most common annotation need for bug reports (labeling UI elements, adding "this should say X" notes).
- **No session organization:** Screenshots land in a default folder with no association to the bug they document. Testers must mentally track which screenshots go with which bug.
- **Metadata is manual:** Software version, meeting ID, environment info, and console output must all be gathered and transcribed by hand.
- **Context switching kills flow:** Stopping to create a ticket every time you find a bug disrupts the testing session. But if you wait until the end, you forget details.
- **Bug descriptions are inconsistent:** Without a template or AI assistance, bug reports vary wildly in quality and completeness.

### What Success Looks Like

A tester presses Print Screen when they see something wrong. They capture a few screenshots, paste a URL, jot a quick note. They press F4 and move on. At the end of the session, they review 20-30 well-organized bugs with AI-drafted descriptions, make edits, and have organized folders ready for ticket creation in under 30 minutes total — including the capture time.

---

## 3. Product Overview

### Product Name

**Unbroken QA Capture**

### What It Is

A desktop tray application that runs alongside the application under test. It manages QA sessions, captures and organizes bug evidence, annotates screenshots, and uses Claude AI to help draft bug reports.

### What It Is Not

- Not a screen recording tool (it orchestrates OS-native tools for that)
- Not a ticketing system (it produces organized output that feeds into your existing ticketing workflow)
- Not a test case manager or test automation tool
- Not a real-time collaboration tool (v1 is single-user)

### Core Value Proposition

1. **Zero-friction capture:** Print Screen starts a bug, F4 ends it. No forms, no dialogs, no context switching.
2. **Automatic organization:** Every screenshot, video, note, and URL is automatically associated with the right bug in the right session folder.
3. **AI-powered descriptions:** Claude reviews your screenshots and notes and drafts a structured bug report. You review and refine.
4. **Solves the Windows annotation gap:** Built-in text, rectangle, oval, and freehand annotation directly on screenshots.
5. **Clean output:** Organized folders per session and per bug, ready to attach to tickets in any system.

---

## 4. User Personas

### Primary: Manual QA Tester (v1 Target)

- Tests desktop and browser-based applications manually
- Finds 10-30 bugs or feedback items per testing session
- Currently uses Windows 11 as primary QA machine
- Has a Claude subscription (Pro or Max) with Claude Code access
- Uses Linear for ticketing with an existing workflow
- Values speed during testing and thoroughness in bug reports
- Technical enough to use hotkeys and understand console output, but not a developer

### Secondary: Product Manager / Product Reviewer (Future)

- Reviews software builds and provides feedback
- Captures feature requests and UX feedback, not just bugs
- Less technical, needs an even simpler capture flow
- May not have Claude access (tool must work without AI features)

---

## 5. User Flow & Interaction Model

### End-to-End Session Flow

#### Phase 1: Start Session

The tester launches Unbroken QA Capture (or it auto-starts with Windows). The app sits in the system tray with a small icon. The tester right-clicks the tray icon and selects "Start Session" or presses the session hotkey (default: **Ctrl+Shift+Q**).

When a session starts:
- A new session folder is created under the configured sessions root directory, named with the current date and a short ID (e.g., `2026-02-13_a3f2/`).
- The Windows Snipping Tool's default save location is programmatically redirected to a temporary capture folder within the session directory via Windows Registry modification.
- The tray icon changes appearance to indicate an active session (e.g., changes color or adds a dot indicator).
- A minimal session status widget appears (configurable: floating mini-bar or just the tray icon). This shows session duration, bug count, and current state.

The tester then opens the application they are testing and begins their QA work.

#### Phase 2: Bug Capture (Repeatable)

The tester encounters a bug, unexpected behavior, or has feedback on a feature. They press **Print Screen**.

When Print Screen is pressed during an active session:
- The OS screenshot mechanism is triggered (Snipping Tool on Windows 11).
- Simultaneously, the app creates a new bug capture entry with an auto-incrementing ID (Bug-01, Bug-02, etc.).
- The app enters **Bug Capture Mode**. The tray icon or status widget updates to show "Capturing Bug-03" (or whatever the current number is).
- The screenshot saved by Snipping Tool lands in the session's capture folder. The file watcher detects it and automatically moves/copies it into the active bug's subfolder.

While in Bug Capture Mode, the tester can:

- **Take more screenshots** (Print Screen again) — additional screenshots are associated with the same bug.
- **Capture video** — using Snipping Tool's video mode or Xbox Game Bar. Video files landing in the capture folder are associated with the current bug.
- **Open the quick notepad** (hotkey, default: **Ctrl+Shift+N**) — a small, always-on-top text panel appears where the tester can paste a meeting URL, type a quick note like "this only happens when 3+ participants are in the call", or paste any other context. The notepad can be dismissed and re-opened without losing content. It is associated with the current bug.
- **Take a console screenshot** — the tester screenshots the developer console of the app under test. During review, Claude will parse this for errors and warnings.
- **Annotate a screenshot** — the tester can open the most recent screenshot (or any screenshot in the current bug) in the annotation overlay to add text labels, rectangles, ovals, or freehand markings.

The tester presses **F4** (or clicks "End Bug Capture" in the app) to close the current bug capture. The app returns to passive QA mode within the active session. The bug entry is saved with all its associated captures and notes.

The tester continues testing. When they find the next issue, they press Print Screen again, and a new bug capture (Bug-02, Bug-03, etc.) begins.

#### Phase 3: Session Notes (Available Anytime)

At any point during an active session (whether in Bug Capture Mode or not), the tester can access a **session-level notepad**. This is separate from the per-bug notepad. It's for errant thoughts, general observations, or ideas that don't belong to a specific bug yet.

Session notes can later be:
- Converted into a new bug entry during review
- Folded into an existing bug's description
- Left as general session observations

The session notepad is accessible via a hotkey (default: **Ctrl+Shift+M**) or from the app's main window.

#### Phase 4: End Session & Review

The tester finishes their QA work and ends the session via the tray icon menu, the app's main window, or the session hotkey (Ctrl+Shift+Q again to toggle).

When a session ends:
- The file watcher stops monitoring.
- The Windows Snipping Tool save location is restored to its original path via the registry.
- The app opens the **Session Review** view.

In Session Review, the tester sees a list of all bugs captured during the session. Each bug card shows:
- Bug ID and type (Bug / Feature / Feedback — can be set during review if not set during capture)
- Thumbnail previews of associated screenshots
- Any notes or pasted URLs from the quick notepad
- A "Generate Description" button (or auto-generated if the setting is enabled)

The tester reviews each bug individually:

1. **Review screenshots:** Thumbnails are visible; clicking opens full-size. The tester confirms the right screenshots are associated. They can see the subfolder path for manual attachment to tickets later.
2. **Review/edit notes:** Any quick notes captured during the bug are shown. The tester can edit or add more.
3. **AI description generation:** The tester clicks "Generate Description" (or it's already auto-generated). The app invokes Claude Code CLI with a focused prompt containing only that bug's screenshots, notes, and metadata. Claude returns a structured bug report draft. The tester reviews it.
4. **Interactive refinement:** If the draft needs changes, the tester can type instructions like "make the steps to reproduce more specific" or "mention that this only happens on the second attempt" and the app sends a scoped follow-up to Claude. Each interaction is a clean, bounded CLI invocation — no accumulated context bloat.
5. **Manual editing:** The tester can also directly edit the description text at any time, bypassing Claude entirely for any given bug.
6. **Finalize:** The tester marks the bug as "ready" and moves to the next one.

Once all bugs are reviewed, the tester can:
- Push to their ticketing system (Linear for v1) using the pluggable integration
- Or simply use the organized folder output with their existing workflow

#### Phase 5: Output

After review, the session folder contains the complete, organized output:

```
Sessions Root (user-configured)/
└── 2026-02-13_a3f2/
    ├── session-notes.md          # Session-level observations
    ├── session-summary.md        # Auto-generated summary of all bugs
    ├── Bug-01/
    │   ├── screenshots/          # All screenshots for this bug
    │   │   ├── capture-001.png
    │   │   ├── capture-001_annotated.png
    │   │   └── capture-002.png
    │   ├── video/                # Any video captures
    │   │   └── recording-001.mp4
    │   ├── notes.md              # Quick notes captured during bug
    │   └── description.md        # Final bug description (AI-generated + edited)
    ├── Bug-02/
    │   ├── screenshots/
    │   │   └── capture-001.png
    │   ├── notes.md
    │   └── description.md
    └── ...
```

Each `description.md` follows a consistent template that maps to the tester's Linear ticket format. The tester can open these files, copy content into tickets, and drag screenshots from the subfolder into the ticket's attachment area.

---

## 6. Application States & State Machine

The application has four distinct states. Transitions are triggered by hotkeys or UI buttons.

### State: Idle

- **Description:** App is running in system tray. No active session.
- **Tray icon:** Default (gray or neutral).
- **Visible UI:** None (tray icon only). Main window available if clicked.
- **Transitions:**
  - Ctrl+Shift+Q → **Active Session**
  - Tray icon → "Start Session" → **Active Session**
  - Main window → "Start Session" button → **Active Session**

### State: Active Session (QA Mode)

- **Description:** Session is running. Tester is actively QA-ing the target application. No bug is currently being captured.
- **Tray icon:** Active indicator (green dot or color change).
- **Visible UI:** Minimal status widget (optional) showing session time and bug count. Main window available.
- **File watcher:** Running, monitoring session capture folder.
- **Registry:** Snipping Tool output redirected to session folder.
- **Transitions:**
  - Print Screen → **Bug Capture Mode** (creates new bug, triggers OS screenshot)
  - Main window → "New Bug Capture" button → **Bug Capture Mode** (without triggering screenshot, for cases like starting a video capture)
  - Ctrl+Shift+Q → **Session Review** (ends active capture phase)
  - Ctrl+Shift+M → Opens session notepad (remains in Active Session state)

### State: Bug Capture Mode

- **Description:** A bug is actively being captured. All new screenshots, videos, and notes associate with this bug.
- **Tray icon:** Capture indicator (red dot or pulsing).
- **Visible UI:** Status widget shows "Capturing Bug-03" or similar. Quick notepad accessible via hotkey.
- **File watcher:** Running. New files auto-associate with current bug.
- **Transitions:**
  - Print Screen → Takes another screenshot (stays in Bug Capture Mode, same bug)
  - Ctrl+Shift+N → Opens/closes quick notepad for current bug (stays in Bug Capture Mode)
  - F4 → **Active Session** (closes current bug capture)
  - Main window → "End Bug Capture" button → **Active Session**
  - Ctrl+Shift+Q → **Session Review** (auto-closes current bug capture, ends session)

### State: Session Review

- **Description:** Active capture is over. Tester is reviewing, editing, and finalizing bugs.
- **Tray icon:** Review indicator.
- **Visible UI:** Full review window with bug list, detail view, Claude AI interaction.
- **File watcher:** Stopped.
- **Registry:** Snipping Tool output restored to original path.
- **Transitions:**
  - "Push to Linear" → Executes ticketing integration, returns to **Idle**
  - "Close Session" → Returns to **Idle** (output folders preserved)
  - "Resume Session" → Returns to **Active Session** (if tester wants to capture more)

---

## 7. Architecture

### Technology Stack

| Layer | Technology | Rationale |
|---|---|---|
| Desktop Shell | Tauri 2 (Rust + WebView2) | ~5MB installed, native OS access via Rust, web UI via WebView2. No Electron overhead. |
| Frontend | Vue 3 + TypeScript + Quasar Framework | Composition API throughout. Quasar provides a rich, pre-built component library (buttons, dialogs, drawers, lists, toolbars, notifications) that dramatically reduces UI build time. Supports custom brand theming via Sass variables. |
| State Management | Pinia | Official Vue 3 state management. TypeScript-native, devtools integration, modular stores. |
| Annotation Engine | HTML5 Canvas via Fabric.js | Mature canvas library with text, shapes, and freehand built-in. Active maintenance. |
| File Watching | Rust `notify` crate | Cross-platform filesystem event monitoring. Low overhead, reliable. |
| Local Database | SQLite via `rusqlite` | Session and bug metadata index. Lightweight, no server process, survives app restarts. |
| AI Integration | Claude Code CLI (subprocess) | Uses tester's existing Claude subscription. No API key or per-token cost. Detailed in Section 11. |
| Ticketing | Pluggable integration layer | Linear first (via existing Claude.md workflow). Interface designed for Jira/others. Detailed in Section 12. |

### Architecture Principles

1. **Thin orchestrator, not monolith.** The app coordinates OS-native tools (Snipping Tool, file system) and external services (Claude CLI, Linear). It doesn't rebuild what the OS already does well.
2. **Capture is fast and dumb, review is smart and structured.** During capture, the app just collects files and short text. All organization, AI processing, and ticket formatting happens during review when the tester has time.
3. **Scoped AI invocations.** Every Claude CLI call is a focused, single-purpose invocation with only the data needed for that task. No long-running conversations, no context accumulation, no memory leaks from growing prompt sizes.
4. **Platform abstraction from day one.** The Rust backend uses a trait-based platform abstraction for OS-specific operations (registry, screenshot invocation, file paths). v1 implements Windows. macOS implements the same traits later.
5. **Graceful degradation.** If Claude CLI is not installed or not authenticated, all AI features are disabled and the tool functions as a manual capture-and-organize tool. If the ticketing integration is not configured, output is still organized folders with markdown files.

### System Architecture (Logical)

```
┌─────────────────────────────────────────────────────────┐
│                    TAURI SHELL                           │
│                                                         │
│  ┌──────────────────────┐  ┌─────────────────────────┐  │
│  │   RUST BACKEND       │  │   VUE 3 + QUASAR        │  │
│  │                      │  │   FRONTEND               │  │
│  │  Session Manager ◄──────► Session UI / Tray        │  │
│  │  File Watcher    ◄──────► Bug Card List            │  │
│  │  Registry Bridge ◄──────► Annotation Canvas        │  │
│  │  Claude CLI Bridge◄──────► Review & Edit View      │  │
│  │  Ticketing Bridge ◄──────► Settings Panel          │  │
│  │  SQLite Store     │  │  │  Quick Notepad           │  │
│  │                      │  │                          │  │
│  └──────────┬───────────┘  └──────────────────────────┘  │
│             │                                            │
└─────────────┼────────────────────────────────────────────┘
              │
    ┌─────────┼──────────┐
    │         │          │
    ▼         ▼          ▼
┌────────┐ ┌──────┐ ┌────────────┐
│ OS     │ │Claude│ │ Ticketing  │
│ Tools  │ │ CLI  │ │ System     │
│ (Snip) │ │      │ │ (Linear)   │
└────────┘ └──────┘ └────────────┘
```

Communication between Rust backend and Vue frontend is via Tauri IPC (invoke commands + event system). The Rust backend owns all filesystem, registry, subprocess, and database operations. The Vue/Quasar frontend owns all UI rendering and user interaction.

---

## 8. Module Specifications

Each module below is designed as an independent, parallelizable work unit for the swarm build system. Modules communicate through well-defined Tauri IPC commands and TypeScript interfaces. A swarm agent can build and test any module in isolation using the provided interface contracts.

---

### Module 1: Session Manager

**Owner:** Rust backend  
**Priority:** P0 — Critical path. All other modules depend on this.  
**Estimated complexity:** Medium (3-5 dev hours with AI assist)  
**Dependencies:** None (foundational module)  

#### Responsibilities

- Manage session lifecycle: start, end, resume.
- On session start: create timestamped session folder under the user-configured sessions root. Folder naming: `YYYY-MM-DD_<short-id>/` where short-id is a 4-character alphanumeric hash for uniqueness when multiple sessions happen on the same day.
- On session start: insert session record into SQLite with `session_id`, `started_at`, `status='active'`, `folder_path`.
- On session end: update SQLite record with `ended_at`, `status='ended'`. Trigger registry restoration (Module 2). Stop file watcher (Module 2).
- On session resume: re-activate a session that was ended but not yet finalized. Re-start file watcher and re-redirect registry.
- Register global hotkeys via Tauri's `global-shortcut` plugin. Default hotkeys (all configurable in settings):
  - **Ctrl+Shift+Q:** Toggle session (start if idle, end if active)
  - **Print Screen:** Start new bug capture + trigger OS screenshot (only when session is active)
  - **F4:** End current bug capture (only when in bug capture mode)
  - **Ctrl+Shift+N:** Toggle quick notepad for current bug
  - **Ctrl+Shift+M:** Toggle session notepad
- Manage tray icon state changes (idle, active, capturing, reviewing).
- Emit session lifecycle events to frontend via Tauri event system: `session:started`, `session:ended`, `session:resumed`, `bug:capture-started`, `bug:capture-ended`.
- Maintain the "current active bug" pointer so other modules know where to associate incoming data.
- Create bug capture entries with auto-incrementing IDs scoped to the session (Bug-01, Bug-02, ...).
- Create the subfolder structure for each bug when bug capture starts.

#### Tauri IPC Commands

```rust
#[tauri::command]
fn start_session(sessions_root: String) -> Result<Session, String>

#[tauri::command]
fn end_session(session_id: String) -> Result<SessionSummary, String>

#[tauri::command]
fn resume_session(session_id: String) -> Result<Session, String>

#[tauri::command]
fn start_bug_capture(session_id: String) -> Result<BugCard, String>

#[tauri::command]
fn end_bug_capture(session_id: String, bug_id: String) -> Result<BugCard, String>

#[tauri::command]
fn get_active_session() -> Result<Option<Session>, String>

#[tauri::command]
fn get_session_bugs(session_id: String) -> Result<Vec<BugCard>, String>

#[tauri::command]
fn update_bug_card(session_id: String, bug_id: String, updates: BugCardUpdate) -> Result<BugCard, String>
```

#### Tauri Events (Backend → Frontend)

```
session:started     { session_id, folder_path, started_at }
session:ended       { session_id, bug_count, duration }
bug:capture-started { session_id, bug_id, bug_number }
bug:capture-ended   { session_id, bug_id, capture_count }
```

#### Acceptance Criteria

1. Hotkey starts/stops session from any foreground application without stealing focus from the app under test.
2. Session folder created with correct naming format within 200ms of session start.
3. SQLite records persist across app restarts. If the app crashes during an active session, the session is recoverable (status remains 'active', can be resumed or ended on next launch).
4. Tray icon visually distinguishes all four states: idle, active session, bug capture, review.
5. No hotkey collisions with common applications (VS Code, Chrome, Slack, Teams — tested manually).
6. Bug IDs are sequential within a session and never duplicated.
7. Session folder structure matches the spec in Section 10 exactly.

#### Error Handling

- If sessions root folder doesn't exist or isn't writable: show error dialog with path and ask user to fix in settings.
- If SQLite database is corrupted: attempt recovery, create new database if necessary, log the error.
- If a hotkey is already registered by another application: log a warning, notify the user via tray notification, and suggest an alternative hotkey in settings.

---

### Module 2: OS Capture Bridge

**Owner:** Rust backend (platform-specific modules)  
**Priority:** P0 — Core capture functionality  
**Estimated complexity:** Medium-High (4-6 dev hours, platform-conditional code)  
**Dependencies:** Module 1 (Session Manager)  

#### Responsibilities

- Define a platform abstraction trait `CaptureBridge` with Windows and macOS implementations. v1 implements `WindowsCaptureBridge` only.
- **Windows: Snipping Tool Registry Redirect**
  - On session start: read the current value of the Windows Registry key `HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Explorer\User Shell Folders\{B7BEDE81-DF94-4682-A7D8-57A52620B86F}`. Cache this original value in memory and in SQLite (for crash recovery).
  - Write the session's capture folder path as the new value.
  - On session end (or app crash via Rust `Drop` trait): restore the original registry value.
  - This ensures that when the user presses Win+Shift+S (or Print Screen triggers snipping), the Snipping Tool saves directly to the session folder.
- **Windows: Print Screen Interception**
  - When the global hotkey for Print Screen fires (registered in Module 1), the app must:
    1. First, create the new bug entry (or confirm we're adding to an existing bug capture).
    2. Then, trigger the Snipping Tool programmatically. On Windows 11 this can be done via: launching `ms-screenclip:` URI, or spawning `SnippingTool.exe`, or sending the `Win+Shift+S` key combination via the Windows API.
    3. The user completes their snip in the Snipping Tool as normal.
    4. The file watcher (below) detects the saved file and associates it.
  - **Important:** The app should NOT steal focus or show any UI during this process. The user should go directly from pressing Print Screen to seeing the Snipping Tool overlay.
- **File Watcher**
  - Use the Rust `notify` crate to watch the session capture folder for new files.
  - Supported file types: `.png`, `.jpg`, `.jpeg`, `.gif`, `.mp4`, `.webm`, `.mkv`.
  - When a new file is detected:
    1. Determine if there is an active bug capture. If yes, move the file to that bug's `screenshots/` or `video/` subfolder (based on extension).
    2. If no active bug capture, hold the file in a `unsorted/` folder within the session. During review, the user can assign it to a bug or discard it.
    3. Emit a Tauri event: `capture:file-detected { bug_id, file_path, file_type }`.
  - File watcher must be started when a session starts and stopped when a session ends.
  - File watcher must handle rapid sequential files (user takes 3 screenshots in quick succession).
- **macOS stub (v2):**
  - The `CaptureBridge` trait should be defined now with a `MacCaptureBridge` struct that returns `Err("macOS support coming in v2")` for all methods. This ensures the abstraction is in place.
  - macOS implementation will use the `screencapture` CLI tool with `-i` flag for interactive capture and output directed to the session folder.

#### Platform Abstraction Trait

```rust
pub trait CaptureBridge {
    /// Redirect OS screenshot tool output to the given folder path.
    /// Returns the original path so it can be restored later.
    fn redirect_screenshot_output(&self, target_folder: &Path) -> Result<PathBuf, CaptureError>;

    /// Restore OS screenshot tool output to the original path.
    fn restore_screenshot_output(&self, original_path: &Path) -> Result<(), CaptureError>;

    /// Programmatically trigger the OS screenshot tool.
    fn trigger_screenshot(&self) -> Result<(), CaptureError>;

    /// Start watching a folder for new capture files.
    fn start_file_watcher(&self, folder: &Path, sender: Sender<CaptureEvent>) -> Result<WatcherHandle, CaptureError>;

    /// Stop the file watcher.
    fn stop_file_watcher(&self, handle: WatcherHandle) -> Result<(), CaptureError>;
}
```

#### Tauri IPC Commands

```rust
#[tauri::command]
fn redirect_snip_path(session_folder: String) -> Result<String, String>  // returns original path

#[tauri::command]
fn restore_snip_path(original_path: String) -> Result<(), String>

#[tauri::command]
fn trigger_screenshot() -> Result<(), String>

#[tauri::command]
fn start_file_watcher(session_folder: String) -> Result<(), String>

#[tauri::command]
fn stop_file_watcher() -> Result<(), String>
```

#### Tauri Events (Backend → Frontend)

```
capture:file-detected  { bug_id, file_path, file_type, timestamp }
capture:file-sorted    { bug_id, file_path, destination_path }
```

#### Acceptance Criteria

1. After session start, pressing Win+Shift+S in Snipping Tool saves the capture to the session folder (not the default Screenshots folder). Verified within 1 second of save.
2. Registry is **always** restored on session end, even if the app crashes. Rust `Drop` trait implementation must be tested by killing the process and verifying registry state.
3. If the app crashes and registry is not restored (e.g., system power loss), the next app launch detects the stale redirect and restores it before doing anything else.
4. File watcher detects new files within 500ms of creation and emits the correct event.
5. Files are moved to the correct bug subfolder based on type (screenshots/ or video/).
6. Three screenshots taken within 2 seconds are all correctly detected and associated.
7. No orphaned file watcher threads after session end (verified via thread count monitoring).
8. File watcher does not lock files (other applications can still access them).

#### Error Handling

- Registry read/write failures: log error, notify user via tray notification, fall back to manual file organization (user must move screenshots themselves).
- File watcher fails to start (permission issue): log error, notify user, session proceeds without auto-association.
- File move fails (file locked by Snipping Tool): retry with exponential backoff (100ms, 200ms, 400ms) up to 3 times, then leave in place and notify user.

---

### Module 3: Annotation Overlay

**Owner:** Vue 3 / Quasar frontend  
**Priority:** P0 — Solves the core Windows annotation gap  
**Estimated complexity:** Medium (3-4 dev hours with Fabric.js)  
**Dependencies:** Module 2 (OS Capture Bridge) for file detection events  

#### Responsibilities

- Provide a canvas-based image annotation editor that opens over a captured screenshot.
- Built with Fabric.js loaded into an HTML5 Canvas element.
- **Annotation tools (v1 — matching the user's current workflow):**
  - **Text labels:** Click to place, type text, drag to reposition. Configurable font size (small/medium/large presets: 14px, 20px, 28px). Default color: red.
  - **Rectangles:** Click and drag to draw. Stroke only (no fill) so the underlying screenshot is visible. Configurable stroke width.
  - **Ovals:** Click and drag to draw. Stroke only. Configurable stroke width.
  - **Freehand draw:** For circling elements or underlining text on screen. Smooth line rendering. Configurable stroke width.
- **Color palette:** Preset colors — red (#FF3B30), yellow (#FFCC00), blue (#007AFF), green (#34C759), white (#FFFFFF), black (#000000). Plus a custom color picker for edge cases.
- **Stroke width:** Three presets — thin (2px), medium (4px), thick (8px). Accessible via toolbar buttons, not a slider.
- **Undo/redo:** Ctrl+Z / Ctrl+Shift+Z. Minimum 50-step history.
- **Save behavior (configurable in settings):**
  - Option A (default): Save annotated version alongside original as `filename_annotated.png`. Original is preserved.
  - Option B: Overwrite original with annotated version.
- **Window behavior:**
  - Opens as a separate Tauri window (not in the main app window).
  - Frameless or minimal frame. Always on top.
  - Sized to fit the screenshot at 1:1 pixel ratio, capped at 90% of the display's viewport dimensions. If the screenshot is larger, it's displayed at a reduced zoom with the ability to scroll/pan.
  - Toolbar at the top or side with tool buttons, color swatches, stroke width, undo/redo, save, cancel.
- **Trigger:**
  - Automatically opens when a new screenshot file is detected by the file watcher during bug capture mode (configurable — can be turned off so the user manually opens annotations).
  - Can also be opened manually from the bug card in the review view by clicking on a screenshot thumbnail.
- **Keyboard shortcuts within the annotation window:**
  - T: Text tool
  - R: Rectangle tool
  - O: Oval tool
  - D: Freehand draw tool
  - Ctrl+Z: Undo
  - Ctrl+Shift+Z: Redo
  - Ctrl+S: Save and close
  - Escape: Cancel and close (discard annotations)
  - Delete/Backspace: Delete selected annotation object

#### Component Structure

```
AnnotationOverlay.vue (Tauri Window)
├── AnnotationToolbar.vue
│   ├── ToolSelector (Text, Rectangle, Oval, Freehand)
│   ├── ColorPalette (6 presets + custom picker via QColor)
│   ├── StrokeWidthSelector (thin, medium, thick via QBtnToggle)
│   ├── UndoRedoButtons (QBtn)
│   └── SaveCancelButtons (QBtn)
└── AnnotationCanvas.vue
    └── Fabric.js Canvas (loaded with screenshot as background)
```

#### Tauri IPC Commands

```rust
#[tauri::command]
fn open_annotation_window(image_path: String) -> Result<(), String>

#[tauri::command]
fn save_annotated_image(image_path: String, annotated_data: Vec<u8>, save_mode: SaveMode) -> Result<String, String>
```

#### Acceptance Criteria

1. Text can be placed anywhere on the screenshot by clicking. Text is editable after placement (double-click to edit). Text renders at the configured size and color with good contrast against any background (consider adding a subtle text shadow or background).
2. Rectangles and ovals render with correct aspect ratios. Stroke-only (no fill) so the underlying screenshot is visible. Can be resized after drawing by dragging handles.
3. Freehand draw produces smooth lines (Fabric.js's freeDrawingBrush with appropriate smoothing). Suitable for circling UI elements and underlining text.
4. Undo/redo works for all operations, minimum 50 steps.
5. Annotated image saves at the original screenshot's resolution. No quality loss from the canvas rendering.
6. Annotation window opens in under 300ms from trigger (file detection event or manual open).
7. Save produces a PNG file with annotations baked in (flattened — not layered).
8. Cancel discards all annotations and leaves the original file untouched.
9. Window does not appear in screenshots taken by Snipping Tool (it should be hidden or minimized when the user takes another screenshot during bug capture — however, this is a stretch goal for v1 and may not be feasible).

#### Error Handling

- If the image file is corrupted or unreadable: show error message in the annotation window, offer to close.
- If save fails (disk full, permissions): show error dialog with the specific error, offer to save to an alternative location.
- If Fabric.js fails to initialize: fall back to displaying the image without annotation capability, log the error.

---

### Module 4: Metadata Collector

**Owner:** Rust backend + Vue 3 / Quasar frontend  
**Priority:** P1 — Important but session works without it  
**Estimated complexity:** Low-Medium (2-3 dev hours)  
**Dependencies:** Module 1 (Session Manager)  

#### Responsibilities

- **Meeting ID / URL field:** Part of the quick notepad UI (Module 7). When the user pastes a URL or meeting ID during bug capture, it's stored as structured metadata on the bug card, not just as freeform text. The field should be smart enough to detect URLs (starts with `http://` or `https://`) and store them as clickable links.
- **Meeting ID persistence:** Within a session, the most recently entered meeting ID pre-populates for subsequent bugs. The user can change it per-bug if they switch workspaces during the session.
- **Software version detection:**
  - **Automatic (best effort):** On session start, attempt to read the version of the app under test. Strategies: read the foreground window's title bar (some apps include version), check for a version file in a configured path, or read from a known registry key. This is configurable per-project in settings.
  - **Manual fallback:** A text field in the session setup or bug card where the user types the version.
  - **Persistence:** Like meeting ID, the version persists across bugs within a session.
- **Environment auto-detection:** On session start, automatically collect:
  - OS name and version (e.g., "Windows 11 23H2 Build 22631")
  - Display resolution and DPI scaling (e.g., "2560x1440 @ 150%")
  - Available RAM and CPU (via Rust `sysinfo` crate)
  - The name of the foreground application when the session started
- **Console screenshot tagging:** When a screenshot is captured, the user can tag it as a "console capture" via the quick notepad or a button in the bug card. This tag tells the review system to send this specific screenshot to Claude for error/warning extraction (Module 5).
- **Structured metadata object:** All metadata is stored as a structured object on each bug card, not embedded in freeform text. This enables consistent ticket templates.

#### Metadata Schema

```typescript
interface BugMetadata {
  meetingId: string | null;         // Pasted URL or ID
  softwareVersion: string | null;   // Detected or manual
  environment: Environment;         // Auto-detected
  consoleCaptures: string[];        // File paths of screenshots tagged as console
  customFields: Record<string, string>;  // For future extensibility
}

interface Environment {
  os: string;                       // "Windows 11 23H2 Build 22631"
  displayResolution: string;        // "2560x1440"
  dpiScaling: string;               // "150%"
  ram: string;                      // "32 GB"
  cpu: string;                      // "Intel i7-13700K"
  foregroundApp: string;            // "MyApp v2.1.0"
}
```

#### Tauri IPC Commands

```rust
#[tauri::command]
fn detect_environment() -> Result<Environment, String>

#[tauri::command]
fn get_foreground_window_title() -> Result<String, String>

#[tauri::command]
fn set_bug_metadata(session_id: String, bug_id: String, metadata: BugMetadata) -> Result<(), String>

#[tauri::command]
fn get_bug_metadata(session_id: String, bug_id: String) -> Result<BugMetadata, String>
```

#### Acceptance Criteria

1. Meeting ID entered during Bug-01 pre-populates for Bug-02. User can override per-bug.
2. Environment data collected silently on session start within 500ms. No user interaction required.
3. Console screenshots tagged via UI button. Tag persists and is visible in review.
4. All metadata fields are editable by the user in the review phase before ticket creation.
5. URLs pasted into the meeting ID field are detected and stored as clickable links.
6. Metadata is persisted to SQLite — survives app restart during an active session.

#### Error Handling

- Environment detection fails (sysinfo crate issue): populate with "Unknown" and log error. Don't block session start.
- Foreground window title detection fails: leave blank, user can enter manually.

---

### Module 5: Claude CLI Bridge

**Owner:** Rust backend (subprocess management) + Vue 3 / Quasar frontend (UI)  
**Priority:** P1 — Major differentiator, but tool functions without it  
**Estimated complexity:** Medium (4-6 dev hours — subprocess management, output parsing, error handling)  
**Dependencies:** Claude Code CLI installed and authenticated on the user's machine  

**This module is the AI integration layer. It is critical that this module is well-specified because it uses an unconventional integration pattern (CLI subprocess) rather than a direct API call.**

#### How Claude Code CLI Works

Claude Code CLI (`claude`) is a command-line tool that Anthropic provides for interacting with Claude using a user's existing Claude subscription (Pro, Max, Team, Enterprise). It authenticates via OAuth — the user runs `claude` once, logs in via browser, and the OAuth token is cached in the OS keychain.

The key advantage: **no API key, no per-token billing.** The user's subscription covers usage.

From a Tauri/Rust app, we invoke `claude` as a subprocess, pass it a prompt via stdin or command-line arguments, and capture the output from stdout.

#### CLI Invocation Pattern

The Claude Code CLI supports a non-interactive mode ideal for programmatic use:

```bash
# Basic invocation with a text prompt
echo "Your prompt here" | claude --print --output-format json

# With an image file (for vision tasks like console parsing)
claude --print --output-format json --file /path/to/screenshot.png "Extract all errors and warnings from this console screenshot. Return as JSON."

# With multiple files
claude --print --output-format json --file /path/to/img1.png --file /path/to/img2.png "Describe the bug shown in these screenshots."
```

**Flags explained:**
- `--print` (or `-p`): Non-interactive mode. Takes the prompt, sends it to Claude, prints the response, and exits. No REPL, no conversation history.
- `--output-format json`: Returns structured JSON output that can be parsed programmatically.
- `--file`: Attaches a file (image, text) to the prompt for Claude to analyze.

**Important:** Each `--print` invocation is a **standalone, stateless request**. There is no conversation memory between invocations. This is exactly what we want for scoped, small-context AI tasks.

#### Rust Subprocess Management

```rust
use std::process::Command;
use serde::Deserialize;

#[derive(Deserialize)]
struct ClaudeResponse {
    result: String,
    // other fields depending on --output-format json structure
}

/// Invoke Claude CLI with a text-only prompt.
/// Returns Claude's response as a string.
fn invoke_claude_text(prompt: &str) -> Result<String, ClaudeError> {
    let output = Command::new("claude")
        .args(["--print", "--output-format", "json"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            // Write prompt to stdin
            if let Some(mut stdin) = child.stdin.take() {
                use std::io::Write;
                stdin.write_all(prompt.as_bytes())?;
            }
            child.wait_with_output()
        })
        .map_err(|e| ClaudeError::SpawnFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ClaudeError::ExecutionFailed(stderr.to_string()));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    // Parse JSON output
    Ok(stdout.to_string())
}

/// Invoke Claude CLI with image files attached.
fn invoke_claude_with_images(prompt: &str, image_paths: &[&str]) -> Result<String, ClaudeError> {
    let mut cmd = Command::new("claude");
    cmd.args(["--print", "--output-format", "json"]);

    for path in image_paths {
        cmd.args(["--file", path]);
    }

    cmd.arg(prompt);

    let output = cmd.output()
        .map_err(|e| ClaudeError::SpawnFailed(e.to_string()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ClaudeError::ExecutionFailed(stderr.to_string()));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}
```

#### AI Task Definitions

Each AI task is a well-defined, scoped invocation. The app constructs the prompt, invokes Claude CLI, and parses the response. No task ever sees data from another task.

##### Task 1: Console Screenshot Parsing

**When:** During review, for any screenshot tagged as "console capture."  
**Input:** Single screenshot image file.  
**Prompt:**
```
You are analyzing a screenshot of a developer console from a desktop application.
Extract all visible errors, warnings, and relevant log entries.
Return your response as JSON in this exact format:
{
  "errors": ["error message 1", "error message 2"],
  "warnings": ["warning message 1"],
  "info": ["relevant info message 1"],
  "raw_text": "complete text visible in the console"
}
If you cannot read the console clearly, set the relevant fields to empty arrays and
explain in a "notes" field what was unclear.
```
**Output parsing:** JSON → `ConsoleParseResult` struct.  
**Timeout:** 15 seconds. On timeout, skip and mark as "parse failed — manual review needed."

##### Task 2: Bug Description Drafting

**When:** During review, when the user clicks "Generate Description" on a bug card (or auto-triggered if enabled in settings).  
**Input:** All screenshots for the bug (image files), the user's quick notes (text), and the bug metadata (meeting ID, version, environment).  
**Prompt:**
```
You are a QA engineer writing a bug report. Based on the provided screenshots
and notes, write a clear, structured bug report.

User's notes: {notes_text}

Software version: {version}
Environment: {environment_summary}
Meeting ID: {meeting_id}

{If console parse results exist: "Console errors found: {console_errors}"}

Write the bug report in this format:

## Summary
One-sentence description of the bug.

## Steps to Reproduce
Numbered steps based on what you can infer from the screenshots and notes.
If steps are unclear, note what you can determine and mark uncertainties.

## Expected Behavior
What should happen (if inferrable).

## Actual Behavior
What actually happened, based on the visual evidence.

## Additional Context
Any relevant details from the metadata, console errors, or notes.

Keep the description concise and actionable. Do not invent information not
supported by the evidence provided.
```
**Output parsing:** Raw markdown text → stored as `description.md` for the bug.  
**Timeout:** 30 seconds (multiple images may take longer).

##### Task 3: Description Refinement (Interactive)

**When:** During review, when the user types a refinement instruction after reviewing a draft.  
**Input:** The current description text + the user's instruction.  
**Prompt:**
```
Here is a bug report draft:

---
{current_description}
---

The QA reviewer wants the following change:
{user_instruction}

Rewrite the bug report incorporating this feedback. Keep the same format.
Output only the revised bug report, no commentary.
```
**Output parsing:** Raw markdown text → replaces current `description.md`.  
**Timeout:** 15 seconds.

##### Task 4: Session Summary Generation

**When:** At the end of session review, before finalizing.  
**Input:** List of all bug titles and one-sentence summaries.  
**Prompt:**
```
Here are the bugs found in this QA session:

{For each bug: "Bug-{id}: {title} — {one_sentence_summary}"}

Write a brief session summary (3-5 sentences) highlighting:
- Total bugs found
- Most critical issues
- Any patterns or themes across bugs
- Overall assessment of the build quality

Keep it concise and professional.
```
**Output parsing:** Raw text → stored as `session-summary.md`.  
**Timeout:** 15 seconds.

#### Tauri IPC Commands

```rust
#[tauri::command]
async fn check_claude_cli_available() -> Result<bool, String>
// Runs `claude --version` and checks for success

#[tauri::command]
async fn check_claude_cli_authenticated() -> Result<bool, String>
// Runs a minimal test prompt to verify auth

#[tauri::command]
async fn parse_console_screenshot(image_path: String) -> Result<ConsoleParseResult, String>

#[tauri::command]
async fn generate_bug_description(
    screenshot_paths: Vec<String>,
    notes: String,
    metadata: BugMetadata
) -> Result<String, String>

#[tauri::command]
async fn refine_bug_description(
    current_description: String,
    user_instruction: String
) -> Result<String, String>

#[tauri::command]
async fn generate_session_summary(
    bug_summaries: Vec<BugSummary>
) -> Result<String, String>
```

**All commands are `async` because they spawn subprocesses that may take several seconds.**

#### Acceptance Criteria

1. `check_claude_cli_available` correctly detects whether `claude` is on PATH and returns true/false.
2. `check_claude_cli_authenticated` correctly detects auth status without causing an error dialog.
3. Console parsing returns structured JSON with errors/warnings arrays within 15 seconds for a typical console screenshot.
4. Bug description generation produces a well-formatted markdown description within 30 seconds.
5. Description refinement correctly incorporates user feedback without losing existing content.
6. Each CLI invocation is fully independent — no state leaks between invocations.
7. If Claude CLI is not available or not authenticated, all AI features gracefully degrade: buttons are disabled with a tooltip explaining "Claude CLI not configured — see Settings", and the user can write descriptions manually.
8. Subprocess is properly cleaned up on timeout (process killed, resources freed).
9. Total subprocess count never exceeds 1 concurrent Claude invocation (queue additional requests).

#### Error Handling

- Claude CLI not found on PATH: set global flag `claude_available = false`. Disable all AI buttons in UI. Show setup instructions in Settings panel.
- Claude CLI not authenticated: set flag `claude_authenticated = false`. Show "Run `claude` in your terminal to log in" message in Settings panel.
- Invocation timeout (15s or 30s depending on task): kill subprocess, return error to frontend, show "AI generation timed out — try again or write manually" message.
- Malformed JSON from console parsing: fall back to raw text output, store as unstructured string.
- Claude returns an error (rate limit, server issue): return error to frontend with Claude's error message, offer retry button.

---

### Module 6: Ticketing Integration (Linear)

**Owner:** Rust backend + Vue 3 / Quasar frontend  
**Priority:** P1 — Primary output mechanism, but organized folders work without it  
**Estimated complexity:** Medium (3-4 dev hours)  
**Dependencies:** Modules 1-5 (consumes their output)  

#### Responsibilities

This module is designed as a **pluggable integration layer**. The v1 implementation supports Linear, but the interface is abstract enough to add Jira or a bespoke system later.

For v1, the Linear integration works with the user's existing Claude.md workflow rather than making direct API calls. This means:

- **The tool does NOT call the Linear API directly in v1.** Instead, it produces structured output (organized folders + markdown descriptions) that the user feeds into their existing Linear workflow (which uses Claude with a project-specific Claude.md file that knows their Linear structure).
- **The integration module's job for v1 is:**
  1. Format each bug's `description.md` according to a configurable ticket template that matches the user's Linear conventions.
  2. Generate a `tickets-ready.md` file that contains all bugs formatted and ready for the user's existing workflow.
  3. Provide a UI button that opens the session folder in the file explorer so the user can access screenshots for manual attachment.
  4. Provide a UI button to copy a single bug's formatted description to the clipboard for quick paste into Linear.

##### Future: Direct API Integration (v2+)

The pluggable interface is designed so that a future version can:
- Authenticate with Linear via OAuth or API key.
- Create issues via the Linear GraphQL API.
- Upload attachments programmatically.
- Sync status back (show Linear issue URL in the app after creation).

#### Integration Interface (Pluggable)

```typescript
interface TicketingIntegration {
  name: string;                                    // "linear", "jira", etc.
  isConfigured(): Promise<boolean>;                // Check if integration is ready
  formatBugForTicket(bug: BugCard): Promise<string>; // Format bug as ticket-ready text
  formatSessionForExport(session: Session): Promise<string>; // Format all bugs
  createTicket?(bug: BugCard): Promise<TicketResult>;  // Optional: direct API creation (v2)
  createTicketBatch?(bugs: BugCard[]): Promise<TicketResult[]>; // Optional: batch (v2)
}

interface TicketResult {
  success: boolean;
  ticketId?: string;
  ticketUrl?: string;
  error?: string;
}
```

#### Default Ticket Template

The template is stored as a configurable markdown file in the app's settings directory. The user can edit it to match their Linear conventions. Default:

```markdown
## Summary
{bug.title || bug.aiDescription.summary}

## Type
{bug.type}  <!-- Bug | Feature | Feedback -->

## Steps to Reproduce
{bug.description.steps || "See attached screenshots and notes."}

## Expected Behavior
{bug.description.expected || "N/A"}

## Actual Behavior
{bug.description.actual || "See attached screenshots."}

## Environment
- **Software Version:** {bug.metadata.softwareVersion}
- **OS:** {bug.metadata.environment.os}
- **Display:** {bug.metadata.environment.displayResolution} @ {bug.metadata.environment.dpiScaling}
- **Meeting ID:** {bug.metadata.meetingId}

## Console Output
{bug.metadata.consoleErrors || "No console errors captured."}

## Attachments
Screenshots and video are in: `{bug.folderPath}`
{for each capture: "- {capture.fileName} ({capture.type})"}
```

#### Tauri IPC Commands

```rust
#[tauri::command]
fn format_bug_as_ticket(session_id: String, bug_id: String) -> Result<String, String>

#[tauri::command]
fn format_session_export(session_id: String) -> Result<String, String>

#[tauri::command]
fn copy_bug_to_clipboard(session_id: String, bug_id: String) -> Result<(), String>

#[tauri::command]
fn open_bug_folder(session_id: String, bug_id: String) -> Result<(), String>

#[tauri::command]
fn open_session_folder(session_id: String) -> Result<(), String>
```

#### Acceptance Criteria

1. Each bug's `description.md` follows the configured ticket template exactly.
2. `tickets-ready.md` in the session folder contains all bugs formatted sequentially, each separated by a clear divider.
3. "Copy to clipboard" produces properly formatted markdown that pastes correctly into Linear's description field.
4. "Open folder" opens the correct folder in Windows File Explorer.
5. The ticket template is user-editable via a text file in the settings directory. Changes take effect on next format operation without app restart.
6. If no template customization exists, the default template is used.
7. The `TicketingIntegration` interface is clean enough that a Jira or custom implementation can be added as a separate module without modifying existing code.

---

### Module 7: Frontend Shell & UI

**Owner:** Vue 3 / Quasar frontend  
**Priority:** P0 — User-facing orchestration layer  
**Estimated complexity:** Medium-High (5-7 dev hours)  
**Dependencies:** All backend modules (consumes their IPC commands and events)  

#### Framework Notes

The entire frontend is built with **Vue 3 + TypeScript** using the **Composition API** (`<script setup lang="ts">`) exclusively. No Options API. State management uses **Pinia** stores. The UI component library is **Quasar Framework**, which provides pre-built, customizable components for the majority of the UI needs.

**Quasar components to leverage (non-exhaustive):**
- `QLayout`, `QDrawer`, `QPage` — Main app layout with collapsible sidebar for bug list
- `QToolbar`, `QBtn`, `QBtnToggle`, `QBtnGroup` — Session controls, annotation tools
- `QCard`, `QCardSection` — Bug cards in the list and detail views
- `QList`, `QItem`, `QItemSection` — Bug card list, settings lists
- `QDialog` — Confirmations, first-run wizard steps
- `QInput`, `QField` — Text inputs for notes, meeting ID, descriptions
- `QEditor` — Rich text/markdown editor for bug descriptions
- `QSplitter` — Side-by-side bug list + detail in review mode
- `QImg`, `QCarousel` — Screenshot thumbnails and gallery view
- `QChip`, `QBadge` — Bug type labels, status indicators
- `QSelect` — Bug type dropdown, ticketing integration selector
- `QTab`, `QTabPanels` — View switching in compact mode
- `QNotify` — Toast notifications for file detection, session events, errors
- `QStepper` — First-run setup wizard
- `QMenu` — Tray icon context menu items
- `QColor` — Color picker in annotation tools
- `QSpinner` — Loading states during Claude CLI invocations
- `QTooltip` — Hotkey hints, disabled feature explanations
- `QToggle`, `QOptionGroup` — Settings toggles

**Quasar brand customization:**
The app uses Quasar's Sass variable system to apply Unbroken Technology brand colors. The `quasar.variables.scss` file defines:
```scss
$primary: #1A1A2E;    // Or Unbroken Technology brand primary
$secondary: #4A90D9;  // Accent blue
$accent: #4A90D9;
$positive: #34C759;
$negative: #FF3B30;
$warning: #FF9500;
$info: #5AC8FA;
$dark: #1A1A2E;
```
These can be swapped to exact brand colors at any time without changing component code.

#### Responsibilities

This module is the entire Vue 3 / Quasar frontend. It provides the UI for all user interactions and listens to backend events via Tauri's event system to update Pinia store state.

#### Window Architecture

The app uses multiple Tauri windows:

1. **Main Window:** The primary app window. Contains the session controls, bug card list, review interface, and settings. Can be minimized to tray. This is the window the user sees when they click the tray icon.
2. **Annotation Window:** A separate, frameless, always-on-top window that opens when annotating a screenshot (Module 3). This is a distinct Tauri window, not a modal within the main window.
3. **Quick Notepad:** A small, always-on-top floating panel triggered by Ctrl+Shift+N during bug capture. This can be either a separate window or a small overlay within the main app — whichever is simpler to implement and less intrusive.

#### UI Screens / Views

##### 1. Tray Icon & Context Menu

The tray icon is the app's persistent presence. Right-click opens a context menu:

- **When Idle:** "Start Session", "Open App", "Settings", "Quit"
- **When Active Session:** "End Session", "Current: Bug-05 capturing", "Open App", "Quit"
- **When Bug Capture:** "End Bug Capture (F4)", "End Session", "Open App"
- **When Review:** "Open Review", "Quit"

Left-click on tray icon opens/focuses the main window.

##### 2. Main Window — Idle State

When no session is active, the main window shows:
- A prominent "Start Session" button.
- A list of recent past sessions with date, bug count, and status (ended, synced).
- A link to Settings.
- App branding: "Unbroken QA Capture" with logo.

##### 3. Main Window — Active Session

When a session is active, the main window shows:
- Session status bar: timer (duration), bug count, current state indicator ("QA Mode" or "Capturing Bug-05").
- A list of bugs captured so far in this session, each as a compact card showing:
  - Bug ID (Bug-01, Bug-02, ...)
  - Thumbnail of first screenshot (if any)
  - Number of captures (e.g., "3 screenshots, 1 video")
  - Brief notes preview (first 50 characters)
  - Status indicator (capturing, captured)
- Buttons:
  - "New Bug Capture" (alternative to Print Screen — for when the user wants to start capturing without taking a screenshot immediately, e.g., to start a video recording first)
  - "End Bug Capture" (alternative to F4)
  - "End Session" (goes to review)
- Session notepad area: a collapsible text area at the bottom for session-level notes. Always accessible, doesn't interfere with bug capture.

**Important:** This window should be designed to be small enough to sit alongside the app under test without obscuring it. Recommended default size: 400px wide, 600px tall. Resizable.

##### 4. Quick Notepad (Bug Capture Mode)

A small floating panel that appears via Ctrl+Shift+N:
- Meeting ID / URL field (pre-populated from last bug if available)
- Free-text notes area
- "Tag next screenshot as console" toggle
- "Done" button (dismisses notepad, stays in bug capture mode)

Size: approximately 300x200px. Always on top. Can be dragged. Disappears on "Done" or Ctrl+Shift+N again.

##### 5. Main Window — Session Review

After ending a session, the main window expands to a review layout:

**Left panel: Bug list**
- All bugs in the session as cards (same as active session view but with review-specific actions).
- Each card shows: ID, thumbnail, capture count, status (needs review / reviewed / ready).
- Click a card to select it and show details in the right panel.

**Right panel: Bug detail**
- **Screenshots gallery:** Grid of thumbnails. Click to view full-size. Click "Annotate" to open annotation window (Module 3). Shows which folder the screenshots are in.
- **Video list:** If any videos captured, listed with filenames and durations.
- **Notes:** Editable text area with the notes captured during bug capture.
- **Metadata display:** Meeting ID, software version, environment info. All editable.
- **Console parse results:** If any console screenshots were tagged and parsed, the extracted errors/warnings are shown here.
- **Bug type selector:** Bug / Feature / Feedback radio buttons.
- **Description section:**
  - "Generate Description" button → invokes Claude CLI (Module 5, Task 2). Shows a loading spinner during generation.
  - The generated (or manually written) description in an editable markdown text area.
  - "Refine with Claude" button + text input for instructions → invokes Module 5, Task 3.
  - Full manual editing at any time.
- **Actions:**
  - "Mark as Ready" — moves to the next bug.
  - "Copy to Clipboard" — copies formatted ticket content.
  - "Open Bug Folder" — opens the bug's folder in File Explorer.
  - "Delete Bug" — removes from session (with confirmation).

**Bottom bar:**
- "Generate All Descriptions" — batch generate for all bugs that don't have descriptions yet.
- "Generate Session Summary" — creates `session-summary.md`.
- "Export to Linear" — runs the ticketing integration, generates `tickets-ready.md`, opens session folder.
- "Resume Session" — go back to active capture mode.
- "Close Session" — finalize and return to idle.

##### 6. Settings Panel

Accessible from main window or tray menu. Organized in sections:

- **General:**
  - Sessions root folder (with "Browse" button)
  - Launch on Windows startup (checkbox)
  - Minimize to tray on close (checkbox)
- **Hotkeys:**
  - Start/end session: default Ctrl+Shift+Q
  - New bug capture: default Print Screen
  - End bug capture: default F4
  - Quick notepad: default Ctrl+Shift+N
  - Session notepad: default Ctrl+Shift+M
  - Each with a "Record hotkey" button for customization
- **Annotation:**
  - Auto-open annotation on screenshot capture (checkbox, default: on)
  - Save mode: alongside original / overwrite (radio)
  - Default color (color picker)
  - Default stroke width (thin/medium/thick)
- **AI (Claude):**
  - Claude CLI status indicator: "Available and authenticated ✓" or "Not found — install instructions" or "Not authenticated — run `claude` to log in"
  - Auto-generate descriptions on review (checkbox, default: off)
  - "Test Claude Connection" button
- **Ticketing:**
  - Integration type: Linear (dropdown, for future extensibility)
  - Ticket template editor (opens the template markdown file in default editor, or an inline text editor)
  - Default bug type (Bug / Feature / Feedback)
  - Linear project configuration file path (for Claude.md reference)
- **About:**
  - App version
  - "Unbroken Technology" branding
  - Links: website, support, changelog

#### State Management (Pinia)

All stores use the Composition API style (`defineStore` with `setup` function):

```typescript
// stores/session.ts
import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSessionStore = defineStore('session', () => {
  // State
  const appState = ref<'idle' | 'active_session' | 'bug_capture' | 'review'>('idle')
  const activeSession = ref<Session | null>(null)
  const activeBugId = ref<string | null>(null)
  const bugs = ref<BugCard[]>([])
  const selectedBugId = ref<string | null>(null)

  // Getters
  const activeBug = computed(() =>
    bugs.value.find(b => b.id === activeBugId.value) ?? null
  )
  const selectedBug = computed(() =>
    bugs.value.find(b => b.id === selectedBugId.value) ?? null
  )
  const bugCount = computed(() => bugs.value.length)
  const readyBugCount = computed(() =>
    bugs.value.filter(b => b.status === 'ready').length
  )

  // Actions
  async function startSession() {
    const session = await invoke<Session>('start_session', {
      sessionsRoot: settingsStore.sessionsRoot
    })
    activeSession.value = session
    appState.value = 'active_session'
    bugs.value = []
  }

  async function endSession() {
    if (!activeSession.value) return
    await invoke('end_session', { sessionId: activeSession.value.id })
    appState.value = 'review'
  }

  async function startBugCapture() {
    if (!activeSession.value) return
    const bug = await invoke<BugCard>('start_bug_capture', {
      sessionId: activeSession.value.id
    })
    bugs.value.push(bug)
    activeBugId.value = bug.id
    appState.value = 'bug_capture'
  }

  async function endBugCapture() {
    if (!activeSession.value || !activeBugId.value) return
    await invoke('end_bug_capture', {
      sessionId: activeSession.value.id,
      bugId: activeBugId.value
    })
    activeBugId.value = null
    appState.value = 'active_session'
  }

  function selectBug(bugId: string) {
    selectedBugId.value = bugId
  }

  async function updateBugNotes(bugId: string, notes: string) {
    await invoke('update_bug_card', {
      sessionId: activeSession.value?.id,
      bugId,
      updates: { notes }
    })
    const bug = bugs.value.find(b => b.id === bugId)
    if (bug) bug.notes = notes
  }

  return {
    appState, activeSession, activeBugId, bugs, selectedBugId,
    activeBug, selectedBug, bugCount, readyBugCount,
    startSession, endSession, startBugCapture, endBugCapture,
    selectBug, updateBugNotes
  }
})

// stores/claude.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useClaudeStore = defineStore('claude', () => {
  const available = ref(false)
  const authenticated = ref(false)
  const processing = ref(false)
  const invocationCount = ref(0)

  async function checkStatus() { /* ... */ }
  async function generateDescription(bugId: string) { /* ... */ }
  async function refineDescription(bugId: string, instruction: string) { /* ... */ }
  async function parseConsole(imagePath: string) { /* ... */ }
  async function generateSessionSummary() { /* ... */ }

  return {
    available, authenticated, processing, invocationCount,
    checkStatus, generateDescription, refineDescription,
    parseConsole, generateSessionSummary
  }
})

// stores/ui.ts
import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUiStore = defineStore('ui', () => {
  const quickNotepadOpen = ref(false)
  const sessionNotepadOpen = ref(false)
  const annotationWindowOpen = ref(false)
  const settingsOpen = ref(false)

  function toggleQuickNotepad() {
    quickNotepadOpen.value = !quickNotepadOpen.value
  }
  function toggleSessionNotepad() {
    sessionNotepadOpen.value = !sessionNotepadOpen.value
  }

  return {
    quickNotepadOpen, sessionNotepadOpen, annotationWindowOpen, settingsOpen,
    toggleQuickNotepad, toggleSessionNotepad
  }
})
```

#### Acceptance Criteria

1. All four app states (idle, active session, bug capture, review) have distinct, clear UI representations.
2. Tray icon context menu is accurate to the current state.
3. Bug cards in active session update in real-time as captures are detected (via Tauri events).
4. Review interface handles 30 bugs without performance degradation (smooth scrolling, fast card switching).
5. Quick notepad opens/closes within 100ms of hotkey press.
6. Quick notepad does not appear in screenshots (it should be dismissable quickly, or the user can close it before screenshotting).
7. Description generation shows a QSpinner loading indicator and disables the button during processing.
8. "Copy to Clipboard" produces correctly formatted markdown.
9. All settings are persisted and applied without app restart (except hotkey changes, which may require restart).
10. The app is usable at 400x600px minimum window size.

---

## 9. Data Model

### SQLite Schema

```sql
CREATE TABLE sessions (
    id TEXT PRIMARY KEY,                -- UUID
    started_at TEXT NOT NULL,           -- ISO 8601
    ended_at TEXT,                      -- ISO 8601, null if active
    status TEXT NOT NULL DEFAULT 'active',  -- active, ended, reviewed, synced
    folder_path TEXT NOT NULL,
    session_notes TEXT,                 -- Session-level notepad content
    environment_json TEXT,              -- JSON blob of Environment struct
    original_snip_path TEXT,            -- Cached original Snipping Tool registry value
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE bugs (
    id TEXT PRIMARY KEY,                -- UUID
    session_id TEXT NOT NULL REFERENCES sessions(id),
    bug_number INTEGER NOT NULL,        -- Sequential within session (1, 2, 3...)
    display_id TEXT NOT NULL,           -- "Bug-01", "Bug-02", etc.
    type TEXT DEFAULT 'bug',            -- bug, feature, feedback
    title TEXT,                         -- Set during review
    notes TEXT,                         -- Quick notepad content
    description TEXT,                   -- Final description (AI-generated + edited)
    ai_description TEXT,                -- Raw AI-generated description (preserved for reference)
    status TEXT NOT NULL DEFAULT 'captured',  -- capturing, captured, reviewed, ready
    meeting_id TEXT,
    software_version TEXT,
    console_parse_json TEXT,            -- JSON: parsed console output
    metadata_json TEXT,                 -- JSON: additional metadata
    folder_path TEXT NOT NULL,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE captures (
    id TEXT PRIMARY KEY,                -- UUID
    bug_id TEXT NOT NULL REFERENCES bugs(id),
    session_id TEXT NOT NULL REFERENCES sessions(id),
    file_name TEXT NOT NULL,
    file_path TEXT NOT NULL,            -- Relative to session folder
    file_type TEXT NOT NULL,            -- screenshot, video, console
    annotated_path TEXT,                -- Path to annotated version, if exists
    file_size_bytes INTEGER,
    is_console_capture BOOLEAN DEFAULT FALSE,
    parsed_content TEXT,                -- Claude-extracted text for console captures
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);

-- Indexes
CREATE INDEX idx_bugs_session ON bugs(session_id);
CREATE INDEX idx_captures_bug ON captures(bug_id);
CREATE INDEX idx_captures_session ON captures(session_id);
```

### TypeScript Types

```typescript
interface Session {
  id: string;
  startedAt: string;
  endedAt: string | null;
  status: 'active' | 'ended' | 'reviewed' | 'synced';
  folderPath: string;
  sessionNotes: string;
  environment: Environment;
  bugs: BugCard[];
}

interface BugCard {
  id: string;
  sessionId: string;
  bugNumber: number;
  displayId: string;           // "Bug-01"
  type: 'bug' | 'feature' | 'feedback';
  title: string;
  notes: string;
  description: string;
  aiDescription: string | null;
  status: 'capturing' | 'captured' | 'reviewed' | 'ready';
  meetingId: string | null;
  softwareVersion: string | null;
  consoleParse: ConsoleParseResult | null;
  metadata: BugMetadata;
  captures: Capture[];
  folderPath: string;
  createdAt: string;
  updatedAt: string;
}

interface Capture {
  id: string;
  bugId: string;
  sessionId: string;
  fileName: string;
  filePath: string;
  fileType: 'screenshot' | 'video' | 'console';
  annotatedPath: string | null;
  fileSizeBytes: number;
  isConsoleCapture: boolean;
  parsedContent: string | null;
  createdAt: string;
}

interface ConsoleParseResult {
  errors: string[];
  warnings: string[];
  info: string[];
  rawText: string;
  notes?: string;
}

interface BugMetadata {
  meetingId: string | null;
  softwareVersion: string | null;
  environment: Environment;
  consoleCaptures: string[];
  customFields: Record<string, string>;
}

interface Environment {
  os: string;
  displayResolution: string;
  dpiScaling: string;
  ram: string;
  cpu: string;
  foregroundApp: string;
}
```

---

## 10. Folder Structure & Output Format

### Sessions Root Directory

The user configures a sessions root directory during first-run setup. Default suggestion: `C:\Users\{username}\Documents\UnbrokenQACapture\`

### Session Folder Structure

Each session creates a timestamped folder:

```
{Sessions Root}/
├── 2026-02-13_a3f2/                    # Session folder
│   ├── .session.json                    # Machine-readable session metadata
│   ├── session-notes.md                 # Session-level notepad content
│   ├── session-summary.md               # AI-generated session summary (after review)
│   ├── tickets-ready.md                 # All bugs formatted for ticket creation
│   ├── _captures/                       # Temporary landing zone for Snipping Tool output
│   │   └── (files here are auto-sorted into bug folders by file watcher)
│   ├── Bug-01/
│   │   ├── screenshots/
│   │   │   ├── capture-001.png
│   │   │   ├── capture-001_annotated.png
│   │   │   └── capture-002.png
│   │   ├── video/
│   │   │   └── recording-001.mp4
│   │   ├── notes.md                     # Quick notes captured during this bug
│   │   └── description.md              # Final bug description
│   ├── Bug-02/
│   │   ├── screenshots/
│   │   │   └── capture-001.png
│   │   ├── notes.md
│   │   └── description.md
│   ├── Bug-03/
│   │   └── ...
│   └── _unsorted/                       # Captures not associated with any bug
│       └── (screenshots taken outside bug capture mode)
│
├── 2026-02-14_b7e1/                    # Another session
│   └── ...
```

### File Naming

- **Screenshots:** `capture-{NNN}.png` where NNN is zero-padded sequential (001, 002, 003...) within the bug.
- **Annotated screenshots:** `capture-{NNN}_annotated.png` — same name with `_annotated` suffix.
- **Videos:** `recording-{NNN}.mp4` (or `.webm`, `.mkv` depending on source).
- **Bug folders:** `Bug-{NN}` with zero-padded two-digit number (Bug-01 through Bug-99). If more than 99 bugs, extend to three digits.

### Machine-Readable Metadata

Each session folder contains a `.session.json` file with the complete session data for programmatic access:

```json
{
  "id": "a3f2...",
  "startedAt": "2026-02-13T09:00:00Z",
  "endedAt": "2026-02-13T11:30:00Z",
  "status": "reviewed",
  "environment": { ... },
  "bugs": [
    {
      "displayId": "Bug-01",
      "type": "bug",
      "title": "Video freezes when third participant joins",
      "description": "...",
      "captures": ["screenshots/capture-001.png", "screenshots/capture-002.png"],
      "metadata": { ... }
    }
  ]
}
```

---

## 11. Claude CLI Integration

This section consolidates and expands on the Claude CLI details from Module 5 for implementer reference.

### Prerequisites

1. **Claude Code CLI must be installed** on the user's machine. Installation: `npm install -g @anthropic-ai/claude-code` (or via the Anthropic installer).
2. **Claude Code CLI must be authenticated.** The user runs `claude` once in their terminal, which opens a browser for OAuth login. The token is cached in the OS keychain.
3. **The app detects these prerequisites** on startup and in settings, and provides clear guidance if they're not met.

### Detection Logic (App Startup)

```
1. Run `claude --version` — if exit code 0, CLI is installed. Parse version string.
2. Run `claude --print "test" --output-format json` — if exit code 0, CLI is authenticated.
3. If step 1 fails: set claudeAvailable = false. Show install instructions.
4. If step 1 succeeds but step 2 fails: set claudeAuthenticated = false. Show auth instructions.
5. If both succeed: set both flags true. AI features enabled.
```

### Subprocess Management Rules

1. **Maximum 1 concurrent Claude CLI subprocess.** Additional requests are queued (FIFO).
2. **Every invocation has a timeout.** 15 seconds for text-only tasks, 30 seconds for vision tasks.
3. **On timeout:** Send SIGTERM to the process. Wait 2 seconds. If still running, SIGKILL. Return timeout error to frontend.
4. **Stdout and stderr are captured separately.** Stdout contains the response. Stderr may contain progress or error information.
5. **Exit code 0 = success.** Any other exit code = failure. Parse stderr for error details.
6. **No shell invocation.** Use `Command::new("claude")` directly, not `Command::new("sh").arg("-c").arg("claude ...")`. This avoids shell injection risks.

### Context Isolation

Every `claude --print` invocation is stateless. The app constructs the complete prompt including all necessary context for each call. There is no conversation history, no session memory, and no context accumulation. This is intentional:

- **Prevents context bloat:** A session with 30 bugs generating descriptions would accumulate massive context if using a conversational approach. With stateless invocations, each call uses only the data relevant to that specific task.
- **Prevents cross-contamination:** A description for Bug-12 is never influenced by the content of Bug-03.
- **Makes errors recoverable:** If a call fails, just retry it. No conversation state to reconstruct.

The one exception is **Task 3 (Description Refinement)**, where the current description is passed back as input. But even this is a complete, self-contained prompt — not a conversation continuation.

### Image Handling

When passing screenshots to Claude CLI:
- Use absolute file paths with `--file` flag.
- Claude CLI handles base64 encoding internally — you don't need to encode the images.
- Maximum recommended images per call: 5. For bugs with more screenshots, select the most relevant 5 or batch them.
- Image files must be accessible (not locked by another process) at the time of invocation.

### Cost and Rate Considerations

Claude CLI usage counts against the user's subscription limits (messages per day/hour depending on plan). The app should:
- Track the number of Claude invocations in the current session (display in settings or session summary).
- Avoid unnecessary invocations: don't auto-generate descriptions for bugs that already have user-written descriptions.
- The "Generate All Descriptions" batch button should warn the user with the count: "This will use approximately 15 Claude requests. Continue?"

---

## 12. Ticketing Integration (Pluggable)

### v1: File-Based Linear Workflow

In v1, the integration with Linear works through the user's existing workflow:

1. The app produces organized folders with markdown-formatted bug descriptions.
2. The user has an existing Claude.md file that defines their Linear project structure, labels, and conventions.
3. The user's existing process (likely involving Claude Code or manual copy) creates the Linear tickets using the formatted output.

The app's role is to produce perfectly formatted output that slots into this existing workflow. The user will share their `Claude.md` file with the swarm at build time so the default ticket template can be tailored to match.

### v2+: Direct API Integration

The pluggable interface (defined in Module 6) is designed so that future versions can:
- Authenticate with Linear via API key (stored in OS keychain).
- Query Linear for projects, teams, labels, and workflow states.
- Create issues directly via Linear's GraphQL API.
- Upload attachments.
- Display issue URLs after creation.

Similarly, Jira or other integrations would implement the same `TicketingIntegration` interface.

### Integration Selection in Settings

The settings UI has a "Ticketing" section with:
- A dropdown for integration type (v1: just "Linear (file-based)" and "None").
- Configuration specific to the selected integration.
- For Linear file-based: a path to the Claude.md reference file, and a ticket template editor.

---

## 13. First-Run Setup & Settings

### First-Run Wizard

On first launch (no settings file exists), the app shows a setup wizard using Quasar's `QStepper` component:

**Step 1: Welcome**
- "Welcome to Unbroken QA Capture"
- Brief description of what the app does
- "Get Started" button

**Step 2: Sessions Folder**
- "Where should session data be saved?"
- Folder picker with default suggestion: `C:\Users\{username}\Documents\UnbrokenQACapture\`
- The app creates the folder if it doesn't exist
- Validation: ensure the path is writable

**Step 3: Claude AI Setup (Optional)**
- Check if Claude CLI is installed and authenticated.
- If yes: "Claude AI is ready! AI features are enabled." with a green checkmark.
- If no: "Claude AI enhances bug reports with AI-generated descriptions. To enable:"
  1. "Install Claude Code: `npm install -g @anthropic-ai/claude-code`"
  2. "Authenticate: Run `claude` in your terminal and log in"
  3. "Come back to this screen and click 'Recheck'"
- "Skip for now" option — AI features will be disabled but can be enabled later in settings.

**Step 4: Hotkeys**
- Show the default hotkey assignments with brief explanations.
- Option to customize each one.
- Conflict detection: if a hotkey is already registered, show a warning.

**Step 5: Done**
- "You're all set! Start your first QA session by pressing Ctrl+Shift+Q or clicking the tray icon."
- "Open Settings" link for further customization.

### Settings Persistence

Settings are stored in:
- **Windows:** `%APPDATA%\UnbrokenQACapture\settings.json`
- **macOS (future):** `~/Library/Application Support/UnbrokenQACapture/settings.json`

SQLite database is stored alongside: `%APPDATA%\UnbrokenQACapture\bugcatcher.db`

---

## 14. UI/UX Specifications

### Design Principles

1. **Stay out of the way.** During active testing, the app should be nearly invisible. The tester's focus is on the app under test, not on the QA tool.
2. **Fast transitions.** Every UI action should feel instant. No full-page transitions, no loading screens for local operations.
3. **Information density in review.** During review, the tester needs to see a lot of information quickly. Favor density over whitespace in the review view.
4. **Consistent with Windows 11 design language.** Use Mica/Acrylic materials, rounded corners, Segoe UI Variable font. The app should feel native.

### Color Palette

Primary: Unbroken Technology brand colors (to be provided by the team).
Fallback palette:
- Primary: Deep navy (#1A1A2E)
- Accent: Blue (#4A90D9)
- Success: Green (#34C759)
- Warning: Amber (#FF9500)
- Error: Red (#FF3B30)
- Background: Windows system background
- Text: System default

### Typography

- Primary font: Segoe UI Variable (Windows system font)
- Monospace (for code/console output): Cascadia Code or Consolas
- Size scale: 12px (small/captions), 14px (body), 16px (headings), 20px (page titles)

### Iconography

- Use a consistent icon set (Lucide, Phosphor, or similar)
- Tray icon should be distinctive and readable at 16x16, 24x24, and 32x32
- State colors on tray icon: gray (idle), green (active session), red (bug capture), blue (review)

### Responsive Layout

Quasar's `QLayout` with `QDrawer` handles the adaptive layout natively. The main window should be usable at:
- **Minimum:** 400 x 500 px (compact mode — QDrawer collapsed, tab-based navigation via QTabs for bug list and detail)
- **Comfortable:** 600 x 800 px (review mode — QDrawer as overlay)
- **Full:** 1000 x 800 px (expanded review — QSplitter with persistent QDrawer for bug list, detail in main content area)

Quasar's breakpoint system (`$q.screen.lt.md`, etc.) drives the layout switching automatically.

---

## 15. Performance & Quality Requirements

| Metric | Target | How to Measure |
|---|---|---|
| Installed size | < 50 MB | Tauri bundle size after compilation |
| Idle RAM (tray only) | < 30 MB | Task Manager, no active session |
| Active session RAM | < 80 MB | Task Manager, with 20 captures loaded |
| Review mode RAM | < 150 MB | Task Manager, with 30 bugs and thumbnails |
| Cold startup time | < 2 seconds | From process start to tray icon visible |
| Hotkey response | < 100 ms | Keypress to action initiation |
| File watcher detection | < 500 ms | File creation to Tauri event emission |
| Annotation window open | < 300 ms | From trigger to window visible with image |
| Screenshot auto-sort | < 1 second | File detection to move into bug folder |
| Claude CLI invocation overhead | < 500 ms | From IPC call to subprocess start (excludes Claude processing time) |
| Bug card render (review) | < 50 ms | Click card to detail view populated |
| Session with 30 bugs: review load | < 2 seconds | From entering review to all cards rendered |

### Memory Leak Prevention

- **File watcher cleanup:** Watcher must be stopped and dropped when session ends. Verified by monitoring thread count.
- **Subprocess cleanup:** Every Claude CLI subprocess must be tracked. On session end or app quit, all running subprocesses are terminated.
- **Image thumbnails:** Use lazy loading in the review view. Only generate thumbnails for visible bug cards. Release memory for off-screen thumbnails.
- **Pinia stores:** Use `$reset()` or manual state clearing when returning to idle. Don't accumulate data from multiple sessions in memory.
- **SQLite connections:** Use a single connection pool. Close all connections on app quit.

### Code Quality

- TypeScript strict mode enabled. No `any` types except in third-party library interop.
- Vue 3 Composition API exclusively — no Options API anywhere in the codebase. All components use `<script setup lang="ts">`.
- Rust: use `clippy` with default lints. No `unwrap()` in production code paths — use proper error handling.
- All modules have unit tests covering at least happy path and primary error paths.
- Integration tests for: session lifecycle (start → capture → end → review), file watcher detection, and Claude CLI invocation.

---

## 16. Security Considerations

- **No secrets in plaintext config files.** Claude CLI manages its own auth via OS keychain. Linear API keys (if added in v2) should also use OS keychain via Tauri's keychain plugin.
- **Registry modifications:** The app modifies a user-level registry key (HKCU, not HKLM). This does not require admin privileges. The original value is always cached and restored.
- **Subprocess safety:** Claude CLI is invoked directly (no shell), preventing command injection. Prompts are passed via stdin or as direct arguments, not interpolated into shell commands.
- **Session data:** Stored locally on the user's filesystem. No cloud sync, no telemetry, no data leaves the machine except for Claude CLI calls (which go through Anthropic's standard API).
- **Annotation images:** Processed entirely locally. No external upload for annotation.

---

## 17. Platform Strategy

### v1: Windows 11

- Full implementation of all modules.
- Snipping Tool integration via registry redirect.
- Windows-specific: tray icon behavior, keyboard shortcuts, registry operations.
- Tested on Windows 11 22H2 and later.

### v2: macOS

- Implement `MacCaptureBridge`:
  - Use `screencapture -i` CLI for screenshots (saves to specified path).
  - Use `screencapture -v` for video capture.
  - No registry modification needed — `screencapture` accepts output path as argument.
  - macOS's built-in Markup tool can be used alongside the app's annotation overlay.
- Adapt hotkeys to macOS conventions (Cmd instead of Ctrl).
- Tray icon → menu bar icon.
- Settings path: `~/Library/Application Support/UnbrokenQACapture/`

### Cross-Platform Architecture

The platform abstraction is enforced from v1:

```rust
// src/platform/mod.rs
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "macos")]
mod macos;

pub fn get_capture_bridge() -> Box<dyn CaptureBridge> {
    #[cfg(target_os = "windows")]
    { Box::new(windows::WindowsCaptureBridge::new()) }
    #[cfg(target_os = "macos")]
    { Box::new(macos::MacCaptureBridge::new()) }  // Returns stub errors in v1
}
```

---

## 18. Build Prioritization & Dependency Graph

### Module Dependency Graph

```
Module 1: Session Manager          (no dependencies)
    ↓
Module 2: OS Capture Bridge        (depends on Module 1)
    ↓
Module 3: Annotation Overlay       (depends on Module 2 for file events)
Module 4: Metadata Collector       (depends on Module 1)
Module 5: Claude CLI Bridge        (no module dependencies, needs Claude CLI installed)
Module 6: Ticketing Integration    (depends on Modules 1-5 output)
Module 7: Frontend Shell           (depends on all backend modules)
```

### Recommended Build Order for Swarm

**Phase 1 (Parallel — Foundation):**
- Module 1: Session Manager — **MUST be first.** All other modules depend on its IPC commands and event system.
- Module 5: Claude CLI Bridge — Can be built in parallel since it has no module dependencies. Just needs the Claude CLI installed on the dev machine.

**Phase 2 (Parallel — Core Capture):**
- Module 2: OS Capture Bridge — Depends on Module 1 being complete.
- Module 4: Metadata Collector — Depends on Module 1 being complete.

**Phase 3 (Parallel — Enhancement):**
- Module 3: Annotation Overlay — Depends on Module 2 for file events.
- Module 6: Ticketing Integration — Can be built once the data model is finalized.

**Phase 4 (Integration):**
- Module 7: Frontend Shell — Integrates all backend modules into the UI. Should be started early (Phase 2) with stubs/mocks for backend modules, then wired up as modules become available.

### Swarm Ticket Suggestions

Each module above maps to one or more tickets. Here is a suggested breakdown:

1. **CORE-001:** Project scaffolding — Tauri 2 + Vue 3 + Quasar + TypeScript project setup, Pinia stores skeleton, folder structure, build configuration, `quasar.variables.scss` with brand colors, development tooling.
2. **CORE-002:** Module 1 — Session Manager (Rust backend, SQLite, hotkey registration, tray icon).
3. **CORE-003:** Module 2 — OS Capture Bridge (registry redirect, file watcher, Print Screen trigger).
4. **CORE-004:** Module 3 — Annotation Overlay (Fabric.js canvas, tools, save/load, window management).
5. **CORE-005:** Module 4 — Metadata Collector (environment detection, meeting ID, version tracking).
6. **CORE-006:** Module 5 — Claude CLI Bridge (subprocess management, all 4 task types, error handling).
7. **CORE-007:** Module 6 — Ticketing Integration (template system, formatting, clipboard, folder open).
8. **CORE-008:** Module 7 — Frontend Shell (all views, state management, event listeners, settings panel).
9. **CORE-009:** First-run wizard and settings persistence.
10. **CORE-010:** Integration testing — end-to-end session flow, cross-module communication.
11. **CORE-011:** User guide documentation (see Section 19).
12. **CORE-012:** Packaging and installer — Tauri bundler configuration, Windows installer, auto-update (optional).

---

## 19. Deliverables

### Software Deliverables

1. **Unbroken QA Capture application** — Windows 11 installer (`.msi` or `.exe` via Tauri bundler).
2. **Source code** — Clean, documented, with README for developers.
3. **Automated tests** — Unit tests for all Rust modules and Vue components. Integration tests for the session lifecycle.

### Documentation Deliverables

1. **User Guide** (see below) — A standalone document that ships with the app or is accessible from the Help menu.
2. **Developer README** — How to set up the dev environment (Node.js, Rust, Quasar CLI), build, test, and contribute.
3. **Architecture Decision Records** — Brief notes explaining key technical choices (why Tauri over Electron, why Fabric.js, why CLI over API, etc.).

### User Guide Requirements

The user guide must be produced as a project deliverable. It should be written in Markdown and also compiled to HTML for in-app access. It must cover:

**Getting Started:**
- System requirements (Windows 11, Claude Code CLI for AI features)
- Installation walkthrough with screenshots
- First-run setup wizard walkthrough
- Setting up Claude Code CLI (installing, authenticating, verifying)

**Core Workflow:**
- Starting your first QA session (step by step)
- Capturing your first bug (Print Screen flow)
- Taking multiple screenshots for one bug
- Using the quick notepad to add context
- Ending a bug capture and moving to the next bug
- Using the session notepad for general thoughts
- Ending a session and entering review

**Annotation Guide:**
- Opening the annotation tool
- Using text labels (placing, editing, resizing)
- Drawing rectangles and ovals
- Freehand drawing for circling and underlining
- Changing colors and stroke width
- Undo/redo
- Saving annotated screenshots

**Review & AI Features:**
- Reviewing a bug's screenshots and notes
- Generating an AI description with Claude
- Refining a description with follow-up instructions
- Manually editing descriptions
- Tagging console screenshots for parsing
- Generating a session summary
- Batch description generation

**Output & Ticketing:**
- Understanding the folder structure
- Finding your screenshots and videos
- Using the formatted descriptions with Linear
- Copying ticket content to clipboard
- Customizing the ticket template

**Settings Reference:**
- All settings explained with recommended defaults
- Hotkey customization
- Annotation preferences
- Claude AI configuration and troubleshooting
- Ticketing integration setup

**Troubleshooting:**
- "Claude AI features are disabled" — how to fix
- "Screenshots aren't appearing in the session folder" — registry/file watcher issues
- "Hotkeys aren't working" — conflict resolution
- "The app didn't restore my Snipping Tool save location" — manual recovery steps
- Session recovery after a crash

**Keyboard Shortcuts Reference:**
- Complete table of all hotkeys with descriptions
- How to customize them

---

## 20. Glossary

| Term | Definition |
|---|---|
| **Session** | A QA testing period. Starts when the user begins testing, ends when they're done. Contains multiple bug captures. |
| **Bug Capture** | A single bug, feature request, or feedback item within a session. Contains screenshots, videos, notes, and metadata. |
| **Bug Capture Mode** | The app state when actively collecting evidence for a specific bug. Entered via Print Screen, exited via F4. |
| **QA Mode** | The app state during an active session when no bug is being captured. The tester is testing but hasn't found anything to capture yet. |
| **Quick Notepad** | A small floating panel for jotting notes during bug capture. Accessible via Ctrl+Shift+N. |
| **Session Notepad** | A text area for session-level notes not tied to a specific bug. Accessible via Ctrl+Shift+M. |
| **Annotation Overlay** | The canvas-based screenshot editor for adding text, shapes, and drawings. |
| **Claude CLI** | The Claude Code command-line interface. Used for AI features (description generation, console parsing). |
| **Snipping Tool Redirect** | The technique of changing the Windows Registry to redirect Snipping Tool's default save location to the session folder. |
| **Ticketing Integration** | The pluggable module that formats bug data for ticket creation in Linear, Jira, or other systems. |
| **Sessions Root** | The user-configured directory where all session folders are created. |

---

## Appendix A: Hotkey Reference (Defaults)

| Hotkey | Action | Available In |
|---|---|---|
| Ctrl+Shift+Q | Start / End session | All states |
| Print Screen | Start new bug capture + take screenshot | Active Session, Bug Capture |
| F4 | End current bug capture | Bug Capture Mode |
| Ctrl+Shift+N | Toggle quick notepad | Bug Capture Mode |
| Ctrl+Shift+M | Toggle session notepad | Active Session, Bug Capture |

### Annotation Window Hotkeys

| Hotkey | Action |
|---|---|
| T | Text tool |
| R | Rectangle tool |
| O | Oval tool |
| D | Freehand draw tool |
| Ctrl+Z | Undo |
| Ctrl+Shift+Z | Redo |
| Ctrl+S | Save and close |
| Escape | Cancel and close |
| Delete | Delete selected object |

---

## Appendix B: Claude CLI Command Reference

```bash
# Check if Claude CLI is installed
claude --version

# Non-interactive text prompt
echo "Your prompt" | claude --print --output-format json

# Non-interactive with image file
claude --print --output-format json \
  --file /path/to/image.png \
  "Your prompt about the image"

# Non-interactive with multiple images
claude --print --output-format json \
  --file /path/to/img1.png \
  --file /path/to/img2.png \
  "Your prompt about these images"

# Authentication (user runs this manually once)
claude
# Opens browser for OAuth login
# Token cached in OS keychain
```

---

*End of PRD — Unbroken QA Capture v1.0*
*Unbroken Technology, February 2026*
