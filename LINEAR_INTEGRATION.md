# Linear Integration Guide

This document explains how to use the Linear ticket creation system for QA testing of the Contio MeetingOS application.

## Setup

1. Copy `.env.example` to `.env`:
   ```bash
   cp .env.example .env
   ```

2. Get your Linear API key from https://linear.app/settings/api

3. Add your API key to `.env`:
   ```
   LINEAR_API_KEY=lin_api_YOUR_ACTUAL_KEY_HERE
   ```

## Session Management

Before filing tickets, you need to set up a testing session for the day:

1. Check if a session file exists for today: `testing-sessions/2026-02-16.md` (use current date)
2. If none exists, create one with this information:
   - **Operating System** (e.g., Windows 11, macOS Sonoma 14.5)
   - **Contio MeetingOS Version** (e.g., 0.42.0)
   - **Environment** (Dev | Staging | Beta | Production)
   - Any other relevant session context

3. Create today's screenshot folder if needed: `regression/screenshots/2026-02-16/`

Example session file (`testing-sessions/2026-02-16.md`):
```markdown
# Testing Session - 2026-02-16

- **Operating System**: Windows 11
- **Contio MeetingOS Version**: 0.42.0
- **Environment**: Production
- **Notes**: Testing post-deployment regression
```

## Creating Tickets

### Testing Without API Key (Dry-Run Mode)

You can test the ticket creation system without an API key or with the `--dry-run` flag:

```bash
node create-ticket.mjs '{"title":"Test Ticket","description":"This is a test"}' --dry-run
```

This will output what would be created without actually making the API call.

### Creating a Bug Ticket

1. Gather information:
   - Steps to reproduce
   - Expected vs actual behavior
   - Workaround (if any)
   - Occurrence frequency
   - Impact
   - Meeting ID/link (if applicable)
   - Screenshots

2. Format the ticket using the bug template:

```json
{
  "title": "QA Bug Found - Login fails with Google OAuth",
  "description": "# Description\n\nGoogle OAuth login fails with 500 error on production.\n\n## Workaround\nUse email/password login instead.\n\n# Steps to Reproduce\n**Preconditions**: User must have a Google account linked\n\n1. Navigate to login page\n2. Click \"Sign in with Google\"\n3. Complete Google authentication\n\n## Expected\nUser is logged in and redirected to dashboard\n\n## Actual\n500 error displayed, user remains on login page\n\n# Bug Report Metadata\n\n| Description | Metadata |\n|---|---|\n| **Operating System** | Windows 11 |\n| **Environment** | Production |\n| **Meeting ID** | N/A |\n| **Meeting Link** | N/A |\n| **Contio MeetingOS Version** | 0.42.0 |\n| **Occurrence Frequency** | 100% |\n| **Impact** | Workflow Disruption |\n| **Workaround Difficulty** | Easy |\n\n# Screenshots\nNone provided",
  "priority": 0,
  "assigneeId": "976c10e6-d43b-49f0-9007-e59c88c4624b",
  "labelIds": ["9ac15a53-d87b-4f6c-8360-1870482689f7"],
  "stateId": "aa635d13-f2bb-48f2-a395-2fd15e0b0441"
}
```

3. Create the ticket:

```bash
node create-ticket.mjs '<JSON from step 2>'
```

4. The script returns:
```json
{
  "success": true,
  "id": "issue-id",
  "identifier": "TEAM-123",
  "title": "QA Bug Found - Login fails with Google OAuth",
  "url": "https://linear.app/..."
}
```

### Creating a Feature Request

1. Gather information:
   - Feature description and rationale
   - Workaround (current way to achieve goal, if any)
   - Acceptance criteria
   - Area of application
   - Impact

2. Format the ticket using the feature template:

```json
{
  "title": "[QA Feature Suggestion] Add keyboard shortcut for quick note capture",
  "description": "# Feature Request\n\nAdd Ctrl+N keyboard shortcut to quickly capture notes during meetings.\n\n# Additional Information\nUsers frequently need to jot down quick notes but must navigate through menus, losing focus on the meeting.\n\n## Workaround\nUse the Notes menu, click Add Note button.\n\n# Acceptance Criteria\n- [ ] Ctrl+N opens a quick note capture dialog\n- [ ] Dialog allows text entry without leaving current view\n- [ ] Note is saved to current meeting context\n\n# Feature Metadata\n\n| Description | Metadata |\n|---|---|\n| **Feature Type** | Enhancement |\n| **Area** | Meeting Capture |\n| **Impact** | Added Time |\n| **Workaround Difficulty** | Moderate |\n\n# Screenshots or Video of Workflow\nNone provided",
  "priority": 0,
  "assigneeId": "db3e5058-b690-48c3-9bfb-e11b314e59f3",
  "labelIds": ["90a8f314-886d-4952-ad6c-0fbedf136c43", "9ac15a53-d87b-4f6c-8360-1870482689f7"],
  "stateId": "aa635d13-f2bb-48f2-a395-2fd15e0b0441"
}
```

3. Create the ticket using the same command as bugs.

## Templates

### Bug Template Structure

```markdown
# Description
{1-2 sentence description}

## Workaround
{Steps for workaround OR "No current workaround identified"}

# Steps to Reproduce
**Preconditions**: {environmental setup or app conditions}

1. {step 1}
2. {step 2}
3. {final step = error state}

## Expected
{What should happen}

## Actual
{What actually happened}

# Bug Report Metadata

| Description | Metadata |
|---|---|
| **Operating System** | {from session} |
| **Environment** | {from session} |
| **Meeting ID** | {if applicable, otherwise "N/A"} |
| **Meeting Link** | {if applicable, otherwise "N/A"} |
| **Contio MeetingOS Version** | {from session} |
| **Occurrence Frequency** | {percentage or description} |
| **Impact** | {Confusion, Data Loss, Loss of Trust, Added Time, Missed Meetings, Workflow Disruption, Compliance Risk, Privacy Risk, Security Risk} |
| **Workaround Difficulty** | {Easy, Moderate, Hard, None} |

# Screenshots
{User-provided or "None provided"}
```

**Bug Defaults:**
- **Title Format**: `QA Bug Found - {succinct description}`
- **Assignee**: Adam Clark (`976c10e6-d43b-49f0-9007-e59c88c4624b`)
- **Labels**: `untriaged` (`9ac15a53-d87b-4f6c-8360-1870482689f7`)
- **State**: Backlog (`aa635d13-f2bb-48f2-a395-2fd15e0b0441`)
- **Priority**: 0

### Feature Template Structure

```markdown
# Feature Request
{1-2 sentence summary}

# Additional Information
{Detail about the pain point, concern, or workflow}

## Workaround
{Steps for workaround OR "No current workaround identified"}

# Acceptance Criteria
- [ ] {outcome 1}
- [ ] {outcome 2}

# Feature Metadata

| Description | Metadata |
|---|---|
| **Feature Type** | {Net New Feature OR Enhancement} |
| **Area** | {One of: Login, Application Settings, User Settings, Transcript Processing, Desktop Installation, Meeting Views, Agenda Creation, Meeting Capture, Meeting Summary, Sharing Summaries, Action Items, Calendar Integration, Meeting Metadata, Contio AI} |
| **Impact** | {Confusion, Data Loss, Loss of Trust, Added Time, Missed Meetings, Workflow Disruption, Compliance Risk, Privacy Risk, Security Risk} |
| **Workaround Difficulty** | {Easy, Moderate, Hard, None} |

# Screenshots or Video of Workflow
{User-provided or "None provided"}
```

**Feature Defaults:**
- **Title Format**: `[QA Feature Suggestion] {description based on workflow}`
- **Assignee**: ak@contio.ai (`db3e5058-b690-48c3-9bfb-e11b314e59f3`)
- **Labels**: `Feature-request` (`90a8f314-886d-4952-ad6c-0fbedf136c43`), `untriaged` (`9ac15a53-d87b-4f6c-8360-1870482689f7`)
- **State**: Backlog (`aa635d13-f2bb-48f2-a395-2fd15e0b0441`)
- **Priority**: 0

## Reference Data

### Team
- **Team ID**: `44c86ac8-cb80-4302-9d81-a0a350b2c352`

### Available Labels
- `untriaged`: `9ac15a53-d87b-4f6c-8360-1870482689f7`
- `Feature-request`: `90a8f314-886d-4952-ad6c-0fbedf136c43`
- `pre-launch-fix`: `a36d4bad-084e-48ad-b041-2731e6a93087`
- `pre-launch`: `33ed7671-fb72-4512-a4dc-cfb2df9d24bb`
- `S0`: `1e986cb7-cbd7-4943-8ae0-1f855d29a98f`
- `S1`: `7306e288-f4aa-4477-af89-991d51e57fd9`
- `S2`: `4e28f171-e6f9-44f0-b971-7f07a50f8036`
- `S3`: `baba5b54-e766-459d-8ec3-9c503f88d8c7`
- `S4`: `eaf8f4cb-a39d-4549-b9d4-8b044039c8a3`
- `Testing`: `f51ca324-f8a4-4797-9df6-5d0887aa2987`
- `customer-reported`: `cf2a317c-31b4-45c8-82d0-d3fd59db3f44`

### Available States
- `Backlog`: `aa635d13-f2bb-48f2-a395-2fd15e0b0441`
- `Todo`: `29ae681a-2b90-42c5-a713-c812da8b2d4d`
- `In Progress`: `95dc47ed-d62c-4361-a959-284cd3a4d5ef`
- `In Review`: `79783bb7-a30e-44fc-ad05-bdb691db63e1`

### Team Members
- Adam Clark: `976c10e6-d43b-49f0-9007-e59c88c4624b`
- ak@contio.ai: `db3e5058-b690-48c3-9bfb-e11b314e59f3`
- stephanie@contio.ai: `422d9ecd-3c20-4640-97a8-04a7463a02db`
- Andrew Ely: `ecbfce8a-02e4-46f6-9e0c-3b789ef2d13f`
- Wyatt Cupp: `2f71b336-77db-4050-b72b-bf619e3abd0d`
- Josh Davidson: `8991a7b1-d151-418b-b7ef-644846c05853`
- Mike MacDonald: `606a942f-156a-4ae3-8435-6ae2c08244e2`
- Tom Evans: `b2974f16-3309-487d-b3fb-ce1e70190247`
- Brian: `831b8e9d-b8d4-457f-9e81-3df8bf2b8043`
- Blake: `64b2655b-229b-428e-b3dc-eefc1adcd0fb`
- Angie O'Hara: `a047653d-4094-4cdf-8429-0a535f7849f9`
- Shane McGonigal: `9a376316-d273-473e-b60b-ba9ff66ec4b7`

### Area Categories
| Area | Code | Description |
|---|---|---|
| Login | LGN | Email and Social Logins |
| Application Settings | ASET | Settings that change how MeetingOS works and are controlled by an administrator or owner |
| User Settings | USET | Settings set by an individual user for their own MeetingOS account |
| Transcript Processing | TPP | Processing meeting transcripts either uploaded or post meeting |
| Desktop Installation | INSTALL | Covers tests during the installation process of the desktop software including download, installation and uninstall |
| Meeting Views | VIEWS | Ways to view past and future meetings (Home, In My Workspace, Drafts, Recaps on Hold, Backlog, Search) |
| Backlog and Drafts | DRAFT | Meeting state before a meeting is officially created which includes Backlog items and Meeting Drafts |
| Agenda Creation | AGENDA | Creating, editing, and updating the agenda either manually or using contio.ai |
| Meeting Capture | MEET | Meeting capture |
| Meeting Summary | SUMRY | Meeting Summary creation and editing |
| Sharing Summaries | SHARE | Sharing the meeting summary via copy and paste, email, URL |
| Action Items | ACTION | Action item creation and viewing - anything related to action items specifically |
| Calendar Integration | CAL | Calendar integrations and calendar event connections |
| Meeting Metadata | META | Meeting time, date, title, and other metadata entered during setup is shows and saves accurately for the user |
| Contio AI | AI | Contio AI features and related functionality |

## Tips

1. **Always run in dry-run mode first** to verify your ticket JSON is correctly formatted
2. **Use session files** to avoid repeating metadata in every ticket
3. **Capture screenshots** and save them in the dated folder before filing tickets
4. **Be specific** in bug reproduction steps - the more detail, the easier to fix
5. **Check for duplicates** before filing - search Linear first
