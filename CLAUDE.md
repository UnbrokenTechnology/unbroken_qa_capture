# Agent Operating Manual

This file guides autonomous AI agents working on this project.
Read PROJECT.md for business context and product vision.
The original PRD is in the repository as `Unbroken_QA_Capture_PRD.md`.

## Repository

- **GitHub:** https://github.com/UnbrokenTechnology/unbroken_qa_capture
- **Organization:** Unbroken Technology (`UnbrokenTechnology`)
- **Remotes:** `origin` (GitHub), `swarm` (local agent orchestration)

## Workflow

### Sub-Agents

Use sub-agents **liberally**. Context rot is a real problem — iteration, failed experiments, dead-ends, errors, and other token bloat accumulate and degrade output quality. The ideal workflow is for the top-level agent to be an **orchestrator** that delegates to sub-agents:

1. **Research** (sub-agent) — outputs a problem statement
2. **Explore** (sub-agent) — reads code, outputs relevant file/function pointers
3. **Plan** (sub-agent) — outputs an implementation plan
4. **Implement** (sub-agent) — executes the plan

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
