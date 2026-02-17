# Next Session Startup

## Quick Start

```powershell
# Pull latest agent work and sync to GitHub (kills running app automatically)
powershell -ExecutionPolicy Bypass -File scripts/swarm-pull.ps1

# Or pull AND launch the app immediately
powershell -ExecutionPolicy Bypass -File scripts/swarm-pull.ps1 -Run

# Or manually:
npm run tauri:dev
```

**Note:** The swarm repo has one unpulled commit (ticket-92: Claude CLI detection fix). Running `swarm-pull.ps1` will bring it in.

## What Was Fixed This Session

- **Hotkey double-registration** (commit `0b3ec42`) -- Removed redundant `.register()` call in `src-tauri/src/hotkey.rs`; `on_shortcut()` already registers the hotkey implicitly
- **Tray icon disappearing on window close** (commit `0b3ec42`) -- Stored `TrayIcon` in a `static OnceLock` so it is not dropped when the window closes
- **Folder picker in wizard** (commit `0b3ec42`) -- Added `dialog` and `fs` capabilities to `src-tauri/capabilities/default.json` so the native folder dialog actually opens
- **Frontend hotkey display** (commit `eac013c`, `6c37634`) -- Settings panel and wizard now show correct default hotkeys matching the backend
- **Claude CLI detection** (ticket 92, in swarm) -- Added fallback path checks for `~/.local/bin/claude.exe` and `~/.claude/local/claude.exe` on Windows

## Needs Manual Testing

After pulling latest and running the app, verify:

- [ ] **Folder picker in wizard** -- Click "Browse" in the first-run wizard; a native folder dialog should open
- [ ] **Tray icon persistence** -- Close the main window with X; the system tray icon should remain visible
- [ ] **Hotkeys** -- No "HotKey already registered" errors in the console on startup
- [ ] **Tray menu items** -- Right-click tray icon; verify: Start Session, New Bug, Open Main Window, Settings, Quit all work
- [ ] **Claude CLI detection** -- First-run wizard should show Claude CLI as "installed" (if it is installed)

After testing, resolve the verify tickets:
- Ticket 85 (wizard re-verify) -- pass or fail
- Ticket 87 (tray quit re-verify) -- pass or fail

## Open Tickets

| ID | Type | Assigned | Title |
|----|------|----------|-------|
| 24 | work | -- | Write end-to-end integration tests |
| 25 | verify | human | Manual Windows 11 verification |
| 34 | work | -- | Integrate first-run setup wizard on launch |
| 38 | work | -- | UI polish and performance verification |
| 85 | verify | human | Re-verify: First-run wizard integration |
| 87 | verify | human | Re-verify: System tray quit button fix |
| 93 | work | -- | Close stale verification tickets 85 and 87 |
| 94 | question | human | Review and close verification tickets 85 and 87 |

**Total completed tickets:** 85 of 93 (91%)

## Known Issues Still Open

- **Stale verify tickets 85, 87** -- These are waiting on human pass/fail after the latest round of fixes. Ticket 93/94 exist to track closing them.
- **End-to-end tests (ticket 24)** -- No integration tests have been written yet.
- **Wizard launch integration (ticket 34)** -- Wizard exists but may not auto-launch on first run.
- **UI polish (ticket 38)** -- General polish pass not yet done.
- **Windows 11 verification (ticket 25)** -- Full manual verification on Windows 11 not yet completed.
- **Bug status bar overlaps wizard** -- The ActiveSessionView status bar (`.status-bar` in `ActiveSessionView.vue`) renders on top of the FirstRunWizard overlay. Needs z-index fix or conditional hiding when wizard is active.

## Swarm Commands Reference

```powershell
# Start the agent swarm
powershell.exe -Command "swarm start"

# Stop all agents
powershell.exe -Command "swarm stop"

# Check agent status
powershell.exe -Command "swarm status"

# View agent logs
powershell.exe -Command "swarm logs agent-1"
powershell.exe -Command "swarm logs agent-2"

# Scale agents up or down
powershell.exe -Command "swarm scale 3"

# Pull agent work into your working tree
powershell -ExecutionPolicy Bypass -File scripts/swarm-pull.ps1

# Ticket management
python .swarm/ticket/ticket.py --db .swarm/tickets/tickets.db list
python .swarm/ticket/ticket.py --db .swarm/tickets/tickets.db show <ID>
```
