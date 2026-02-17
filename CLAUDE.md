# Agent Operating Manual

This file guides autonomous AI agents working on this project.
Read PROJECT.md for business context and product vision.
The original PRD is in the repository as `Unbroken_QA_Capture_PRD.md`.

## Tech Stack

- **Framework:** Tauri 2 (Rust backend + WebView frontend)
- **Frontend:** Vue 3 + Quasar UI + TypeScript
- **Build tools:** Vite, vue-tsc, vitest
- **State management:** Pinia
- **Dev command:** `npm run tauri:dev`
- **Build command:** `npm run tauri:build`
- **Tests:** `npm test` (vitest)

## Build Prerequisites (Windows)

- **Node.js** (v20+)
- **Rust** (via rustup)
- **Visual Studio 2022/2026 Build Tools** with "Desktop development with C++" workload (required by Tauri for MSVC linker and Windows SDK)

## Known Issues & Gotchas

- **Tauri icons must be 8-bit PNGs.** Tauri's icon decoder does not support 16-bit PNG bit depth. If icons are regenerated, ensure they are saved as 8-bit RGBA. The `.ico` should also be regenerated from 8-bit sources.
- **Shell scripts must use LF line endings.** All `.sh` files in `.swarm/` run inside Linux containers. Windows CRLF line endings cause `exec: no such file or directory` errors. The `.gitattributes` enforces `eol=lf` for `*.sh`, `*.py`, and `Dockerfile`, but after fixing line endings you may need `git add --renormalize .swarm/` to apply the rule to already-tracked files.
- **Keep the bare repo's main in sync.** Agents push to `.swarm/repo.git`. If your local main is ahead (e.g. after manual commits), run `git push swarm main` so agents can merge their feature branches cleanly.
- **Kill the app before rebuilding.** Cargo cannot overwrite the running exe (`Access is denied, os error 5`). Stop `unbroken-qa-capture.exe` before running `npm run tauri:dev` or `tauri:build`. The `swarm-pull.ps1` script handles this automatically.
- **Port 5173 conflicts.** Vite will fail to start if a previous dev server is still bound to port 5173. Kill the old `node`/`vite` process first.
- **Local commits can diverge from swarm.** If you commit locally while agents are also merging to the swarm's main, `git pull swarm main --ff-only` will fail. Rebase with `git rebase swarm/main`, then force-push to origin with `git push origin main --force-with-lease`.
- **Hotkey double-registration bug.** On startup you'll see "HotKey already registered" errors. This is caused by `hotkey.rs:register_hotkey()` calling both `on_shortcut()` (which implicitly registers) and then `register()` (redundant). The hotkeys still work because `on_shortcut()` succeeds first, but the redundant `.register()` call should be removed.
- **PowerShell execution policy.** Windows may block `.ps1` scripts by default. Always invoke with `powershell -ExecutionPolicy Bypass -File <script>`.
- **NEVER set `font-family` on wildcard selectors (`*`, `:deep(*)`, etc.).** This clobbers `font-family: 'Material Icons'` on icon elements, causing all Quasar icons to render as plain text (e.g. "bug_report" instead of the bug icon). The app's typography font is configured via `$typography-font-family` in `src/quasar-variables.sass` — Quasar applies it properly without overriding icon fonts. If you need to set a font on a specific element, use a scoped class selector, never a wildcard.

## Repository

- **GitHub:** https://github.com/UnbrokenTechnology/unbroken_qa_capture
- **Organization:** Unbroken Technology (`UnbrokenTechnology`)
- **Remotes:** `origin` (GitHub), `swarm` (local agent orchestration)

## Swarm CLI

The `swarm` CLI manages autonomous agent swarms. It must be run via PowerShell on Windows (e.g. `powershell.exe -Command "swarm <command>"`).

### Commands

| Command | Description | Usage |
|---------|-------------|-------|
| `swarm init <dir>` | Initialize a project for agent swarms | `swarm init .` |
| `swarm start` | Spin up agent containers and monitor | `swarm start` |
| `swarm stop` | Shut down all agents and monitor | `swarm stop` |
| `swarm status` | Show running agents and queue summary | `swarm status` |
| `swarm logs <service>` | Tail logs for a specific agent | `swarm logs agent-1` |
| `swarm scale <count>` | Adjust number of agent containers | `swarm scale 3` |
| `swarm regenerate` | Regenerate docker-compose.yml and .swarm/ files | `swarm regenerate` |
| `swarm pull` | Pull latest changes from the swarm bare repo | `swarm pull` |
| `swarm watch` | Watch for new commits and auto-pull | `swarm watch --interval 5` |

### OAuth / Token Management

`swarm start` automatically extracts OAuth credentials from `~/.claude/.credentials.json` (on Windows) or the macOS Keychain. No `.env` file is needed. The flow:

1. `extract_oauth_credentials()` in `swarm.py` reads `~/.claude/.credentials.json`
2. Credentials (accessToken, refreshToken, expiresAt) are passed as env vars to docker-compose
3. `entrypoint.sh` seeds them into `/token/credentials.json` on a shared tmpfs volume
4. `token-refresh.sh` handles automatic renewal when tokens are within 30 min of expiry

**To refresh tokens:** Run `/login` in Claude Code, then `swarm stop && swarm start`.

Source: `C:\Users\steph\Repositories\claude-swarm\swarm\swarm.py` (the swarm CLI)

### Pulling Agent Work

Agents push to a bare repo at `.swarm/repo.git`. The main branch is **`main`**. Agents work on feature branches named `ticket-<N>`.

**Quick pull (recommended):**

```powershell
# Pull latest, sync to GitHub
powershell -ExecutionPolicy Bypass -File scripts/swarm-pull.ps1

# Pull and immediately launch the app
powershell -ExecutionPolicy Bypass -File scripts/swarm-pull.ps1 -Run
```

The script handles: killing the running app (so the exe isn't locked), fetching, fast-forward merge, and pushing to origin. It's idempotent — safe to run anytime.

**Manual commands (if needed):**

```bash
# Pull merged work from main
git pull swarm main --ff-only

# See all agent branches
git --git-dir=.swarm/repo.git branch -v

# Fetch a specific agent branch to inspect it
git fetch swarm ticket-<N>
git diff main swarm/ticket-<N> --stat
```

### Configuration

Config lives in `.swarm/config.json`:

```json
{
  "agents": 2,
  "allowed_tools": "Bash,Read,Write,Edit,Glob,Grep,Task,WebFetch,WebSearch",
  "max_turns": 50,
  "monitor_port": 3000,
  "verify_retries": 2,
  "mem_limit": "8g"
}
```

### Ticket CLI

Agents coordinate work via the `ticket` CLI (`.swarm/ticket/ticket.py`). Database: `.swarm/tickets/tickets.db`.

```bash
# Create a ticket
ticket create "Title" [--description TEXT] [--parent ID] [--assign WHO] [--blocked-by ID] [--created-by WHO]

# List / query
ticket list                             # all non-done tickets
ticket list --status open               # filter by status
ticket show 7                           # full detail with comments
ticket count                            # count of non-done tickets

# Work on tickets
ticket claim-next --agent agent-1       # atomically claim next available
ticket comment 7 "message" --author agent-1
ticket complete 7                       # mark done
ticket unclaim 7                        # release without completing

# Dependencies
ticket block 7 --by 12                  # 7 blocked by 12
ticket unblock 7 --by 12                # remove dependency

# Activity
ticket log                              # last 20 events
```

## Workflow

### Sub-Agents

Use sub-agents **liberally and by default**. Context rot is a real problem — iteration, failed experiments, dead-ends, errors, and other token bloat accumulate and degrade output quality. The ideal workflow is for the top-level agent to be an **orchestrator** that delegates to sub-agents:

1. **Research** (sub-agent) — outputs a problem statement
2. **Explore** (sub-agent) — reads code, outputs relevant file/function pointers
3. **Plan** (sub-agent) — outputs an implementation plan
4. **Implement** (sub-agent) — executes the plan

**When responding to the human operator**, use sub-agents as much as possible. Delegate research, exploration, code reading, multi-file searches, and implementation tasks to sub-agents rather than doing them inline. This keeps the main conversation context clean, reduces token bloat, and produces higher quality results. The top-level agent should focus on orchestration — understanding what the user wants, dispatching sub-agents, and summarizing results.

Begin every sub-agent prompt with `"YOU ARE THE SUB-AGENT"` so it does not attempt to recursively spawn its own sub-agents.

### One Ticket, One Job

**You may only do work for the ticket you have claimed.** Never work on tickets you haven't claimed — other agents will pick them up.

If your ticket is broad or covers multiple unrelated changes, your job is **decomposition, not implementation**:

1. Break the ticket into focused sub-tickets (use `--parent <YOUR_TICKET_ID>`)
2. Mark your ticket as complete — the decomposition *is* the deliverable

The sub-tickets go back to the pool where multiple agents can claim and work on them **in parallel**. This is faster and produces higher quality output than one agent attempting everything in a single context window.

**Rule of thumb:** if a ticket requires changes to unrelated parts of the codebase, it should be multiple tickets.

### Artifacts

Any intermediate artifacts (problem statements, research notes, plans) should be:

- Written to `/tmp` (or another temporary location)
- Cleaned up when the workflow is complete

Do **not** leave `IMPLEMENTATION_PLAN.md`, `RESEARCH_NOTES.md`, or similar files in the codebase. Do not document long-term goals or desired improvements in the codebase — that's what the ticketing system is for.

The codebase holds exactly two things:

1. **Code** — the current state of the system
2. **READMEs** — business context relevant to that code

## Code Organization

### Contain Complexity

Complexity is not the enemy — **uncontained** complexity is. When something is inherently complex, isolate it in its own module behind a simple interface.

*Example: Video transcoding is complex, but `transcode(file, "mp4")` is simple. The complexity exists, but it's contained — callers don't need to understand it.*

### Directory Structure

Use directories for both code organization and **progressive disclosure**. Deeply nested structures allow an agent to read only the READMEs and files relevant to its current task, rather than loading the entire codebase. This enforces separation of concerns and enables documentation at multiple abstraction layers.

Prefer this over a single flat `src/` folder.

### Separation of Concerns

Each file should contain **one level of abstraction** and be responsible for **one thing**. The golden rule (aspirational, not always achievable): any change to behavior should require editing only **one file**. If a change touches many files, the logic wasn't properly encapsulated.

### Reusability

Favor simple, reusable components over complex, monolithic ones. If you need the same code in two places, abstract it into a shared component. Avoid copy-paste duplication.

### Third-Party Dependencies

Third-party libraries are fine, but vendor lock-in is not. Wrap external dependencies in an interface layer so the implementation can be swapped later without rewriting callers.

## Design Guidance

### Patterns

- **Context pattern** — avoid global variables by passing a context object
- **Strategy pattern** — avoid switch statements and complex conditionals by delegating to interchangeable strategy objects
- **Validator / policy iteration** — when logic requires many conditional checks, express them as a list of validator or policy objects and iterate over them

### Correctness

Favor **always-right** solutions over **usually-right** solutions.

- Parse structured data with a proper parser, not a regex that handles 90% of cases
- Use enums or typed state objects instead of stringly-typed status values
- Choose the approach that is correct by construction, not correct by convention

### Testing

Write tests for all code changes. Tests are your proof that the code works, and they prevent future agents from breaking your changes. Tests are run by `verify.sh` on every commit.

- **Test-first when possible.** Write the test before the implementation. This clarifies what "done" looks like before you start coding.
- **Test behavior, not implementation.** Tests should verify what the code does, not how it does it. This makes tests resilient to refactoring.
- **Every bug fix gets a test.** Before fixing a bug, write a test that reproduces it. Then fix the code until the test passes. This prevents regressions.
- **Keep tests fast.** Slow tests discourage running them. Prefer unit tests over integration tests where possible.
- **Match the project's testing patterns.** Check existing tests in the codebase and follow their conventions (framework, file location, naming, assertion style).

Whenever possible, after you have finished your changes perform manual testing of the application (build it, run it, hit API endpoints, and verify behavior matches expectations)

If you say a feature is functional, you should have proof beyond unit tests passing

#### If there are no existing tests to copy conventions from

This means you are likely writing the first tests for this repository. Follow best practices and conventions for the language and framework being used.

#### If there are conflicting conventions

This means that multiple agents simultaneously implemented the "first tests" in the repository and the changes were merged. Create a new ticket (using `ticket --db /tickets/tickets.db create`) for consolidating tests and standardizing their implementation. Afterwards, review `ticket --db /tickets/tickets.db list` to see if there are multiple tickets for consolidating tests (a potential race condition if two agents hit these instructions at once). If there are multiple, THE LOWER TICKET ID GET PRIORITY. Please `complete` any tickets of higher ID.

## Documentation

### README Files

Write README files to document **business decisions and intent** — the "why" that isn't obvious from code.

*Example: A video transcoding module's code tells you what it does. But it doesn't tell you why it exists. A README could explain: "The third-party service we integrate with only supports MP4, but users frequently upload MOV files from iPhones."*

README rules:

- **Scope them to modules.** One global README documenting every business decision pulls in irrelevant context. A README in each module directory means agents only load what's relevant to their task.
- **Keep them small.** A README should capture the rationale for the module's existence and any non-obvious constraints. It is not a tutorial or API reference.
- **Update them when code changes.** A stale README is worse than no README. When you change a module's behavior, update its README to match.

### Why READMEs Matter

All code was written with some goal in mind. If you don't know **why** code exists, it becomes dangerous to modify or remove — maybe it handles an edge case or business requirement you don't understand. READMEs preserve that intent so future agents can ensure compliance with the needs the code was written to address.

## Decision Making

- **Technical decisions** (database, framework, architecture): Make the call, document in a comment
- **Business decisions** (users, monetization, direction): Create a human-assigned blocking ticket

## Responding to Human Feedback

Humans can post comments on your ticket while you're working. These comments appear in a live file whose path is given in your prompt as `LIVE COMMENTS`.

- **Check the file periodically** (every few tool calls) using the Read tool
- If the file is empty, no new comments have been posted — carry on
- If it contains comments, read them carefully and adjust your approach
- **Human comments take priority** over your current plan — if a human asks you to change direction, do so
- After adjusting, post a comment on the ticket acknowledging the feedback: `ticket comment <ID> "Acknowledged: <summary of adjustment>" --author $AGENT_ID`

Even if you miss a comment mid-session, the system will catch it after your session ends and give you a follow-up opportunity to adjust.

## Ticket Types

- **work** (default): Normal work for agents to complete
- **question**: You need human input before continuing. Use `--blocks <YOUR_ID>` to block your current ticket until the human answers.
- **proposal**: Suggesting an improvement. Human will approve/reject. No blocker needed.
- **verify**: Human verification needed (e.g. manual build/test). Human can Pass or Fail — failing creates a fix task for agents.

When creating a human-assigned ticket:
- With `--blocks` or `--blocked-by`: defaults to `question` type
- Without either: defaults to `proposal` type

## Platform-Specific Verification

For some platforms (iOS, Windows-native), `verify.sh` may create `verify` tickets assigned to `human` requesting manual build/test. These use `--block-dependents-of` to ensure downstream tickets stay blocked until the human clicks Pass. If the human clicks Fail, a fix task is created for agents with the failure reason, and the cycle repeats. If you see "Manual build/test needed" tickets in the queue, they are not for agents.

## Dependencies

`--blocked-by <ID>` means "this new ticket cannot start until ticket <ID> is done." Create foundational tickets first, then dependent tickets with `--blocked-by`.

If your current ticket depends on unfinished work, run `ticket block <YOUR_ID> --by <PREREQUISITE_ID>` — this automatically releases your ticket back to the pool. Once the prerequisite is done, your ticket becomes claimable again.
