# Unbroken QA Capture — User Guide

## Getting Started

### System Requirements

Unbroken QA Capture is designed for Windows 11 and requires the following:

**Required:**
- **Operating System:** Windows 11 (64-bit)
- **Disk Space:** Approximately 50 MB for application installation
- **Memory:** Minimum 4 GB RAM (8 GB recommended for optimal performance)
- **Display:** 1920x1080 or higher resolution recommended

**Optional (for AI Features):**
- **Claude Code CLI:** Enables AI-powered bug description generation, console screenshot parsing, and session summaries
  - Requires an active Claude.ai subscription
  - Installation instructions provided in the setup wizard

**Note:** Without Claude Code CLI, the application functions as a powerful manual capture-and-organize tool. All AI features gracefully disable when Claude is not available.

---

### Installation

1. **Download the Installer**
   - Download the latest `.msi` or `.exe` installer from the Unbroken Technology distribution site
   - Verify the file is complete (check file size matches the published size)

2. **Run the Installer**
   - Double-click the installer file
   - If Windows SmartScreen appears, click "More info" then "Run anyway"
   - Follow the installation prompts
   - Choose your installation directory (default: `C:\Program Files\UnbrokenQACapture\`)
   - Click "Install" to begin installation

3. **Launch the Application**
   - The installer will offer to launch the application when complete
   - Alternatively, find "Unbroken QA Capture" in your Start menu
   - The application runs in your system tray (look for the Unbroken icon in the taskbar notification area)

---

### First-Run Setup Wizard

When you launch Unbroken QA Capture for the first time, you'll be guided through a setup wizard to configure essential settings. The wizard only appears once — you can modify any of these settings later through the Settings panel.

#### Step 1: Welcome

The welcome screen introduces Unbroken QA Capture and explains what it does:
- Orchestrates OS-native screen capture during QA testing
- Organizes screenshots and videos by bug/issue
- Provides annotation tools (solving the Windows 11 text annotation gap)
- Uses Claude AI to draft structured bug reports
- Creates session-based folders ready for ticketing workflows

Click **"Get Started"** to continue.

#### Step 2: Sessions Folder

This is where all your QA session data will be saved.

**Default Location:**
`C:\Users\{YourUsername}\Documents\UnbrokenQACapture\`

**What gets stored here:**
- Each QA session creates a timestamped folder (e.g., `2026-02-16_a3f2/`)
- Inside each session: organized bug folders with screenshots, videos, notes, and metadata
- All output is portable — you can move, archive, or share these folders freely

**To configure:**
1. Click the folder icon to browse for a different location
2. The application will create the folder if it doesn't exist
3. The wizard validates that the path is writable before continuing

**Recommendation:** Choose a location that:
- Has sufficient disk space (sessions with many screenshots can grow to several GB)
- Is backed up regularly (if your backup system covers Documents, the default is good)
- Is accessible to your ticketing workflow (e.g., synced to cloud storage if you file tickets from multiple machines)

Click **"Next"** to continue.

#### Step 3: Claude AI Setup (Optional)

This step checks if Claude Code CLI is installed and authenticated on your system.

**If Claude is detected:**
- You'll see a green checkmark: "Claude AI is ready! AI features are enabled."
- Click **"Next"** to proceed

**If Claude is not detected:**
- You'll see instructions for enabling AI features
- You can choose to set it up now or skip for later

**To set up Claude Code CLI:**

1. **Install Claude Code CLI**
   ```
   npm install -g @anthropic-ai/claude-code
   ```
   *Requires Node.js 18+ and npm. If you don't have Node.js installed, download it from [nodejs.org](https://nodejs.org/).*

2. **Authenticate Claude**
   - Open a terminal (Windows Terminal, PowerShell, or Command Prompt)
   - Run the command: `claude`
   - Follow the prompts to log in with your Claude.ai account
   - You'll need an active Claude subscription (Pro or Team)

3. **Verify Installation**
   - Click the **"Recheck"** button in the wizard
   - If successful, you'll see the green checkmark

**Skip for now:**
- If you prefer to set up Claude later, click **"Skip for now"**
- AI features will be disabled but can be enabled at any time through Settings
- The application will function fully as a capture-and-organize tool

**What AI features do:**
- **Description Generation:** Claude analyzes your screenshots and notes to draft structured bug descriptions
- **Console Parsing:** Claude extracts error messages and stack traces from console screenshots
- **Session Summaries:** Claude generates an overview of all bugs found in a session
- **Interactive Refinement:** You can ask Claude to revise descriptions with follow-up instructions

Click **"Next"** to continue (or **"Skip for now"** to proceed without AI features).

#### Step 4: Hotkeys

Unbroken QA Capture uses global hotkeys to keep the tool out of your way during active testing. This step shows the default hotkey assignments and allows you to customize them.

**Default Hotkeys:**

| Hotkey | Action | When Available |
|--------|--------|----------------|
| `Ctrl+Shift+Q` | Start/end QA session | Always (when app is running) |
| `Print Screen` | Start bug capture + trigger screenshot | During active session |
| `F4` | End current bug capture | During bug capture mode |
| `Ctrl+Shift+N` | Toggle quick notepad for current bug | During bug capture mode |
| `Ctrl+Shift+M` | Toggle session notepad | During active session |

**Within the Annotation Window:**

| Hotkey | Action |
|--------|--------|
| `T` | Text tool |
| `R` | Rectangle tool |
| `O` | Oval tool |
| `D` | Freehand draw tool |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+S` | Save and close |
| `Escape` | Cancel (discard annotations) |
| `Delete` or `Backspace` | Delete selected annotation |

**To customize hotkeys:**
1. Click on any hotkey to edit it
2. Press your desired key combination
3. The wizard will check for conflicts with other registered global hotkeys
4. If a conflict is detected, you'll see a warning and can choose a different combination

**Best practices:**
- Avoid hotkeys used by applications you frequently test (VS Code, Chrome, Slack, Teams, etc.)
- Keep Print Screen as-is if possible — it's the most intuitive trigger for QA testers
- F4 is chosen because it's rarely used by applications and is easy to reach

Click **"Next"** to continue.

#### Step 5: You're All Set!

The setup is complete. You'll see a summary:

**"You're all set! Start your first QA session by pressing Ctrl+Shift+Q or clicking the tray icon."**

**Next steps:**
- The Unbroken QA Capture icon is now in your system tray (notification area)
- Right-click the icon to access the menu: Start Session, Settings, Help, Exit
- Press `Ctrl+Shift+Q` to start your first session
- The main application window will appear when you start a session or enter review mode

**Additional configuration:**
- Click **"Open Settings"** to explore advanced options:
  - Annotation preferences (default colors, stroke width, save behavior)
  - Auto-annotation triggering (open annotation tool automatically when screenshot is captured)
  - Ticketing integration (Linear API setup)
  - Custom ticket templates
  - AI prompt customization

Click **"Finish"** to close the wizard and begin using Unbroken QA Capture.

---

### Quick Start Guide

Once you've completed the wizard, here's how to capture your first bug:

1. **Start a Session:** Press `Ctrl+Shift+Q` or click the tray icon → "Start Session"
   - The tray icon changes color to indicate an active session (green)
   - A timestamped session folder is created in your configured location

2. **Find a Bug:** Use the application you're testing as normal

3. **Capture Evidence:** When you encounter something worth reporting, press `Print Screen`
   - This starts a new bug capture (Bug-01, Bug-02, etc.)
   - The Windows Snipping Tool appears — capture your screenshot as usual
   - The screenshot automatically saves to the current bug's folder
   - The tray icon changes color to indicate bug capture mode (red)

4. **Add More Evidence:**
   - Take additional screenshots (they all associate with the current bug)
   - Press `Ctrl+Shift+N` to open the quick notepad and jot context ("This should say 'Submit' not 'Send'")
   - If you need to annotate a screenshot, the annotation tool may open automatically (configurable), or you can open it later during review

5. **Finish This Bug:** Press `F4` to end the current bug capture
   - The tray icon returns to active session mode (green)
   - You're ready to find the next bug

6. **Repeat:** Continue testing and capturing bugs (steps 2-5) until your session is complete

7. **End Session:** Press `Ctrl+Shift+Q` again to end the session
   - The application enters **Review Mode**
   - The main window opens showing all captured bugs

8. **Review & Refine:** For each bug:
   - Review screenshots and notes
   - Click **"Generate Description"** to use Claude AI (if enabled)
   - Edit or refine the description as needed
   - Mark the bug as ready for ticketing

9. **Create Tickets:** Use the organized output:
   - Each bug has a folder with all evidence and a formatted description
   - Copy descriptions to your ticketing system (Linear, Jira, etc.)
   - Attach screenshots directly from the bug folders
   - If Linear integration is configured, descriptions include Linear-compatible markdown

---

### What's Next?

- **Core Workflow:** Learn the complete session lifecycle, annotation tools, and review features
- **Annotation Guide:** Master text labels, shapes, colors, and keyboard shortcuts
- **Review & AI Features:** Discover how Claude AI can save you hours of documentation work
- **Settings Reference:** Customize Unbroken QA Capture to match your workflow

For detailed guidance on each topic, see the corresponding sections in this User Guide.

---

## Core Workflow

This section explains the complete session lifecycle, how to capture and organize bugs during testing, and how to use the review interface to prepare bugs for ticketing.

### Session States

Unbroken QA Capture operates as a state machine with four distinct states:

1. **Idle** — No active session. The main window shows recent sessions and a "Start Session" button.
2. **QA Mode** — A session is active, but you're not currently capturing a bug. You're testing the application and looking for issues.
3. **Bug Capture Mode** — You've started capturing a specific bug. Screenshots and notes are associated with this bug.
4. **Review Mode** — The session has ended. You're reviewing bugs, generating descriptions, and preparing them for ticketing.

### Starting a Session

**To start a QA session:**

1. **Via Hotkey (Recommended):** Press `Ctrl+Shift+Q`
2. **Via Tray Icon:** Right-click the system tray icon → "Start Session"
3. **Via Main Window:** Click "Start Session" button (if the main window is open)

**What happens:**
- The application enters **QA Mode**
- The system tray icon changes to green (indicating an active session)
- A timestamped session folder is created in your configured sessions directory (e.g., `2026-02-17_a3f2/`)
- The main window (if open) displays the active session view with a session status bar showing:
  - Session duration timer (starts at 00:00)
  - Bug count (starts at 0)
  - Current state badge ("QA Mode")

**You're now ready to test your application and capture bugs.**

### Capturing a Bug

When you encounter something worth reporting during testing, start capturing evidence:

**Press `Print Screen` to start a new bug capture.**

**What happens:**
1. A new bug is created (Bug-01, Bug-02, etc.)
2. The application enters **Bug Capture Mode**
3. The system tray icon changes to red
4. The session status bar updates to show "Capturing Bug-XX"
5. Windows Snipping Tool opens automatically (Windows 11's native screenshot tool)
6. After you capture the screenshot, it's automatically saved to the bug's folder

**The bug folder structure:**
```
2026-02-17_a3f2/
└── Bug-01/
    ├── screenshot_001.png    ← your first screenshot
    ├── screenshot_002.png    ← additional screenshots for this bug
    └── metadata.json         ← bug metadata (timestamps, status, etc.)
```

### Adding Evidence to a Bug

While in **Bug Capture Mode**, all screenshots you take are automatically associated with the current bug:

**Taking Additional Screenshots:**
- Press `Print Screen` again
- The Snipping Tool opens
- The screenshot is saved to the same bug folder (screenshot_002.png, screenshot_003.png, etc.)

**Adding Notes:**
- The main window displays the active session view where you can see all captured bugs
- Click on the current bug card to view its details
- Edit the notes field directly in the bug card
- Notes are automatically saved as you type

**Editing Metadata:**
- In the active session view, click on a bug card to expand its details
- You can edit:
  - Meeting ID / URL (useful for bugs found during meetings)
  - Software version
  - Free-text notes

### Ending a Bug Capture

When you've gathered sufficient evidence for the current bug, end the capture:

**Press `F4` to end the bug capture.**

**Alternative method:**
- Click "End Bug Capture" button in the main window
- Right-click the tray icon → "End Bug Capture"

**What happens:**
- The bug's status changes from "capturing" to "captured"
- The application returns to **QA Mode**
- The tray icon changes back to green
- You're ready to continue testing and capture the next bug

### Capturing Multiple Bugs

The power of Unbroken QA Capture is organizing multiple bugs in a single session:

1. **Find a bug** → Press `Print Screen` → Take screenshots → Press `F4`
2. **Continue testing** → Find another bug → Press `Print Screen` → Take screenshots → Press `F4`
3. **Repeat** as many times as needed during your testing session

Each bug gets its own folder with its own screenshots and metadata. The session folder keeps everything organized:

```
2026-02-17_a3f2/
├── Bug-01/
│   ├── screenshot_001.png
│   ├── screenshot_002.png
│   └── metadata.json
├── Bug-02/
│   ├── screenshot_001.png
│   └── metadata.json
├── Bug-03/
│   ├── screenshot_001.png
│   ├── screenshot_002.png
│   ├── screenshot_003.png
│   └── metadata.json
└── session.json
```

### Ending a Session

When your testing session is complete, end the session:

**Press `Ctrl+Shift+Q` again to end the session.**

**Alternative methods:**
- Click "End Session" button in the main window
- Right-click the tray icon → "End Session"

**What happens:**
- The session's status changes from "active" to "ended"
- The application enters **Review Mode**
- The main window opens automatically and displays the Session Review interface
- The tray icon returns to its default state

### Session Review Interface

The Session Review interface is where you prepare bugs for ticketing. It uses a master-detail layout:

#### Left Panel: Bug List

- Displays all bugs captured in the session as clickable cards
- Each card shows:
  - **Bug ID badge** (Bug-01, Bug-02, etc.) with color-coded type
  - **Bug type chip** (bug, feature, or feedback)
  - **Title** (editable via the right panel)
  - **Notes snippet** (first 2 lines)
  - **Folder path** (relative path to bug folder)
  - **Screenshot thumbnails** (first 3 screenshots, with "+N more" indicator if there are more)

**To select a bug:** Click on any bug card in the list.

#### Right Panel: Bug Detail

When you select a bug, the right panel displays comprehensive details and editing tools:

**Actions Bar** (top of panel):
- **Copy to Clipboard** — Copies the bug details as markdown format (useful for manual ticketing)
- **Open Folder** — Opens the bug's folder in File Explorer
- **Delete Bug** — Removes the bug from the session (with confirmation dialog)
- **Mark Ready** — Changes the bug status to "ready" (indicates it's finalized and ready for ticketing)

**Bug Type Editor:**
- Toggle between Bug, Feature, or Feedback
- The type affects color coding and can be used as a label when exporting to ticketing systems

**Bug Metadata:**
- Bug ID (display only)
- Status (captured, reviewed, ready)
- Folder path (display only)

**Editable Metadata Fields:**
- Meeting ID / URL (e.g., Zoom meeting link)
- Software version (e.g., "2.4.1")

**Notes Editor:**
- Multi-line text area for free-form notes
- Auto-expands as you type
- Saves automatically

**AI-Generated Description** (if Claude CLI is installed):
- **Generate Description** button — Analyzes screenshots and notes to draft a structured bug description
- **Refine** button (appears after generation) — Opens a dialog where you can provide refinement instructions (e.g., "add more technical details")
- **Save** button — Saves the description to `description.md` in the bug folder
- The description editor is fully editable — you can modify the AI-generated text before saving
- If Claude CLI is not available, this section shows a warning message with setup instructions

**Screenshots Gallery:**
- Grid display of all screenshots for the selected bug
- Click any screenshot to view full-size (opens lightbox dialog)
- **Console capture toggle** (icon button on each thumbnail) — Mark screenshots as console captures (useful for distinguishing UI screenshots from console/log screenshots)
- Screenshot lightbox includes navigation arrows to move between screenshots

#### Bottom Action Bar

The bottom action bar provides session-level operations:

- **Generate All Descriptions** — Runs AI description generation for all bugs in the session (requires Claude CLI)
- **Export to Linear** — Opens the Linear push dialog (see Output & Ticketing section)
- **Resume Session** — Changes the session status back to "active" and returns to QA Mode (useful if you realize you need to capture more bugs)
- **Close Session** — Finalizes the session and returns to Idle state (with confirmation dialog)

### Review Workflow

**Recommended workflow for reviewing a session:**

1. **Select the first bug** from the left panel
2. **Review screenshots** — Click each screenshot to view full-size, verify the issue is clearly visible
3. **Edit metadata** — Update meeting ID, software version if applicable
4. **Edit or add notes** — Clarify what the bug is, steps to reproduce, expected vs. actual behavior
5. **Generate AI description** (if Claude is available):
   - Click "Generate Description"
   - Wait for Claude to analyze screenshots and notes
   - Review the generated description
   - Click "Refine" if you want to adjust it (e.g., "make it more technical" or "add reproduction steps")
   - Click "Save" to write the description to `description.md`
6. **Mark as ready** — Click "Mark Ready" to indicate the bug is finalized
7. **Move to the next bug** — Click the next bug card in the left panel
8. **Repeat steps 2-7** for all bugs

**After all bugs are reviewed:**

- **Export to Linear** (if you use Linear for ticketing) — see Output & Ticketing section
- **Or manually create tickets** — Use "Copy to Clipboard" on each bug, then paste into your ticketing system
- **Close Session** — Click "Close Session" when you're done

### Resuming a Session

If you realize during review that you need to capture more bugs, you can resume the session:

1. Click **"Resume Session"** in the bottom action bar
2. The session returns to **QA Mode**
3. Continue testing and capturing bugs as normal
4. Press `Ctrl+Shift+Q` again to end the session and return to review

### Keyboard Shortcuts Reference (Core Workflow)

| Shortcut | Action | Context |
|----------|--------|---------|
| `Ctrl+Shift+Q` | Start session (from Idle) | Global |
| `Ctrl+Shift+Q` | End session (from QA Mode or Bug Capture Mode) | Global |
| `Print Screen` | Start new bug capture (opens Snipping Tool) | QA Mode |
| `Print Screen` | Take additional screenshot (opens Snipping Tool) | Bug Capture Mode |
| `F4` | End current bug capture | Bug Capture Mode |

**Note:** All global shortcuts can be customized during the first-run setup wizard or in Settings.

### State Transitions Summary

Understanding state transitions helps you navigate the workflow efficiently:

```
Idle
 ↓ [Start Session]
QA Mode
 ↓ [Print Screen]
Bug Capture Mode
 ↓ [F4]
QA Mode
 ↓ [Print Screen] (repeat for more bugs)
Bug Capture Mode
 ↓ [F4]
QA Mode
 ↓ [End Session]
Review Mode
 ↓ [Close Session]
Idle

OR

Review Mode
 ↓ [Resume Session]
QA Mode
```

### Tips for Efficient Bug Capture

**During testing:**
- Keep the main window open on a second monitor (or sized small on the same monitor) to see your bug count in real-time
- Use the session notes area (collapsible notepad at bottom of active session view) for session-level context like "Testing v2.4.1 release candidate"
- Take multiple screenshots per bug from different angles — it's easier to skip screenshots during review than to recreate them later

**During review:**
- Use the "Generate All Descriptions" feature at the start of review if you have Claude CLI — you can refine individual descriptions afterward
- Mark bugs as "ready" as you complete them — this makes it easy to see which bugs still need attention
- Use "Copy to Clipboard" to preview the markdown output before exporting to Linear

---

## Output & Ticketing

Unbroken QA Capture organizes all your QA session data into structured folders that integrate seamlessly with your existing ticketing workflow. This section explains the folder structure, how to use the output with Linear or other ticketing systems, and how to customize the ticket format.

### Understanding the Folder Structure

Each QA session creates a timestamped folder in your configured Sessions Root directory (set during first-run setup, default: `C:\Users\{YourUsername}\Documents\UnbrokenQACapture\`).

#### Session Folder Example

```
UnbrokenQACapture/
└── 2026-02-16_a3f2/                    # Session folder (date + unique ID)
    ├── .session.json                    # Machine-readable session metadata
    ├── session-notes.md                 # Your session-level notepad content
    ├── session-summary.md               # AI-generated session summary (after review)
    ├── tickets-ready.md                 # All bugs formatted for ticket creation
    ├── _captures/                       # Temporary landing zone (auto-sorted)
    ├── Bug-01/
    │   ├── screenshots/
    │   │   ├── capture-001.png
    │   │   ├── capture-001_annotated.png
    │   │   └── capture-002.png
    │   ├── video/
    │   │   └── recording-001.mp4
    │   ├── notes.md                     # Quick notes for this bug
    │   └── description.md               # Final formatted bug description
    ├── Bug-02/
    │   ├── screenshots/
    │   │   └── capture-001.png
    │   ├── notes.md
    │   └── description.md
    ├── Bug-03/
    │   └── ...
    └── _unsorted/                       # Screenshots taken outside bug capture
        └── (rarely used)
```

#### Key Files and Folders

**Session-Level Files:**

- **`.session.json`** — Machine-readable metadata about the entire session (start/end times, environment, bug list). Useful for scripting or integration with other tools.
- **`session-notes.md`** — Your session-level observations captured with `Ctrl+Shift+M`. Use this for general test environment details, meeting notes, or observations that apply to multiple bugs.
- **`session-summary.md`** — AI-generated overview of all bugs found in the session (created when you click "Generate Session Summary" in Review Mode). Provides a high-level summary perfect for team updates or sprint retrospectives.
- **`tickets-ready.md`** — All bugs formatted according to your ticket template, ready for batch import or copy-paste into your ticketing system.

**Bug-Level Folders:**

- **`Bug-01/`, `Bug-02/`, etc.** — Each bug gets its own folder with a sequential ID (Bug-01 through Bug-99, extending to three digits if needed).
- **`screenshots/`** — All screenshots for this bug, named sequentially (`capture-001.png`, `capture-002.png`, etc.). Annotated versions have an `_annotated` suffix.
- **`video/`** — Screen recordings associated with this bug (if captured via external tool during the session).
- **`notes.md`** — Quick notes captured with `Ctrl+Shift+N` during bug capture. This is where you paste meeting URLs, error messages, or context that supplements the screenshots.
- **`description.md`** — The final, formatted bug description ready for ticketing. Generated by Claude AI or written manually during review.

**Special Folders:**

- **`_captures/`** — Temporary landing zone for screenshots from the Windows Snipping Tool. The file watcher automatically moves files from here into the appropriate bug folder. You'll rarely see files here (they move within milliseconds).
- **`_unsorted/`** — Rarely used folder for screenshots captured outside of bug capture mode (e.g., if you take a screenshot during an active session but haven't started a bug capture yet).

#### File Naming Conventions

- **Screenshots:** `capture-{NNN}.png` where `NNN` is zero-padded and sequential within the bug (001, 002, 003...).
- **Annotated screenshots:** `capture-{NNN}_annotated.png` — same base name with `_annotated` suffix.
- **Videos:** `recording-{NNN}.mp4` (or `.webm`, `.mkv` depending on capture source).
- **Bug folders:** `Bug-{NN}` with zero-padded two-digit numbers (Bug-01, Bug-02... Bug-99, then Bug-100 if needed).
- **Session folders:** `YYYY-MM-DD_{ID}` where `ID` is a short unique identifier (e.g., `2026-02-16_a3f2`).

This consistent naming makes it easy to:
- Sort bugs chronologically
- Reference specific screenshots in ticket descriptions
- Script bulk operations (e.g., "attach all screenshots from Bug-05")

---

### Finding Your Session Data

**After ending a session:**

1. The Review Mode window opens automatically, showing all bugs from that session
2. Click **"Open Session Folder"** in the toolbar to open the session folder in File Explorer
3. The session folder contains everything from that testing session — organized and ready to use

**To find past sessions:**

1. Open the Sessions Root folder (configured in Settings → General → Sessions Root Folder)
2. Browse by date — folders are named with the date first for easy sorting
3. Open any `.session.json` file to see a summary of what was captured in that session

**Tip:** Bookmark your Sessions Root folder in File Explorer for quick access. You can also set up automatic backup or cloud sync for this folder to preserve your QA history.

---

### Using Output with Linear

Unbroken QA Capture is designed to work seamlessly with Linear, the modern issue tracking tool. In v1, the integration is file-based — the app produces perfectly formatted output that you copy into Linear manually or via your existing workflow.

#### Workflow: From Bug to Linear Ticket

**Option 1: Copy Individual Bugs**

1. Complete your QA session and enter Review Mode
2. Review a bug, generate or edit its description
3. Click **"Copy to Clipboard"** on the bug detail view
4. Open Linear, create a new issue
5. Paste the formatted description (`Ctrl+V`)
6. Drag screenshots from the bug's `screenshots/` folder into Linear's attachment area
7. Set priority, labels, and assignee as needed

**Option 2: Batch Create from `tickets-ready.md`**

1. After reviewing all bugs in a session, click **"Export to Linear"** in the Review Mode toolbar
2. This generates `tickets-ready.md` in the session folder with all bugs formatted sequentially
3. Open `tickets-ready.md` in your favorite markdown editor
4. Copy each bug's section and paste into Linear one at a time
5. Attach screenshots from the corresponding bug folders

**Option 3: Use Your Existing Linear Workflow**

- If you already have a Claude.md file that defines your Linear project structure, you can reference the session folder in a prompt to Claude Code CLI
- Claude can read the bug descriptions and create Linear tickets via the Linear API using your project conventions
- This leverages your existing automation without requiring changes to Unbroken QA Capture

#### What the Formatted Output Looks Like

The default ticket template produces markdown that maps to Linear's issue format:

```markdown
## Summary
Video freezes when third participant joins

## Type
Bug

## Steps to Reproduce
1. Start a video call with one participant
2. Add a second participant (call continues normally)
3. Add a third participant
4. Observe: video feed freezes for all participants

## Expected Behavior
Video should continue smoothly regardless of participant count.

## Actual Behavior
Video freezes completely when the third participant joins. Audio continues working.

## Environment
- Version: 2.14.3
- Platform: Windows 11
- Meeting ID: abc-defg-hij

## Console Output
TypeError: Cannot read property 'track' of undefined
  at VideoManager.addParticipant (video-manager.js:142)
  at CallController.handleJoin (call-controller.js:89)

## Attachments
Screenshots are in: Bug-03/screenshots/
- capture-001.png (frozen video feed)
- capture-002.png (console error)
- capture-003_annotated.png (annotated UI state)
```

This format works well with Linear's markdown rendering and provides all the information a developer needs to reproduce and fix the bug.

---

### Customizing the Ticket Template

The ticket template is fully customizable to match your team's conventions.

**To edit the template:**

1. Open Settings → Ticketing → Ticket Template
2. Click **"Edit Template"** to open the template file in your default markdown editor
3. Modify the structure, field names, or formatting as needed
4. Save the file — changes take effect immediately

**Template Variables:**

The template uses variable placeholders that are replaced with actual bug data:

- `{bug.title}` — Bug title or AI-generated summary
- `{bug.type}` — Bug, Feature, or Feedback
- `{bug.description.steps}` — Steps to reproduce (if provided)
- `{bug.description.expected}` — Expected behavior
- `{bug.description.actual}` — Actual behavior
- `{bug.metadata.version}` — Application version (if detected)
- `{bug.metadata.environment}` — Platform details (OS, browser, etc.)
- `{bug.metadata.meetingId}` — Meeting ID or URL (if captured in notes)
- `{bug.metadata.consoleErrors}` — Parsed console errors (if any)
- `{bug.folderPath}` — Path to the bug's folder (for attachment reference)
- `{for each capture: "- {capture.fileName} ({capture.type})"}` — List of screenshots/videos

**Example: Simplified Template for Quick Reports**

If your team prefers a simpler format:

```markdown
**{bug.title}**

{bug.description.actual}

Repro: {bug.description.steps}

See screenshots: {bug.folderPath}
```

**Example: Jira-Style Template**

If you're using Jira or a similar tool:

```markdown
h2. Summary
{bug.title}

h2. Description
{bug.description.actual}

h2. Steps to Reproduce
# {bug.description.steps}

h2. Environment
* Version: {bug.metadata.version}
* Platform: {bug.metadata.environment}

h2. Attachments
{for each capture: "!{capture.fileName}!"}
```

**Restoring the Default Template:**

If you want to reset to the default template, delete the custom template file (Settings → Ticketing → Reset to Default) and restart the app.

---

### Copying Ticket Content to Clipboard

The **"Copy to Clipboard"** feature formats a bug according to your ticket template and copies it to the clipboard, ready to paste into Linear, Jira, Slack, or any other tool.

**To use:**

1. In Review Mode, select a bug
2. Click **"Copy to Clipboard"** in the bug detail view
3. The formatted description is now in your clipboard
4. Paste (`Ctrl+V`) into Linear, Jira, or wherever you create tickets

**What gets copied:**

- The full bug description formatted with your ticket template
- Markdown formatting preserved (headings, lists, code blocks)
- File paths to screenshots (you'll still need to attach them manually)

**Note:** In v1, attachments are not automatically uploaded to Linear. You'll need to drag screenshots from the bug folder into Linear's attachment area. Future versions may support direct API integration with automatic attachment upload.

---

### Integration with Other Ticketing Systems

While Linear is the primary target for v1, Unbroken QA Capture is designed to work with any ticketing system that accepts markdown or text input.

**Jira:**

- Use the ticket template to format bugs in Jira's markdown syntax
- Copy to clipboard and paste into Jira's description field
- Attach screenshots manually from the bug folders

**GitHub Issues:**

- GitHub's issue format is markdown-native
- Copy to clipboard and paste directly into a new issue
- Reference screenshots by uploading them to the issue (drag-and-drop)

**Asana, Monday, ClickUp, etc.:**

- These tools generally support rich text or markdown
- Copy to clipboard and paste into the description field
- Customize the ticket template to match each tool's conventions

**Email / Slack:**

- For quick bug reports sent via email or Slack, copy to clipboard works perfectly
- Screenshots can be dragged from the bug folder into email or Slack messages

**Custom Scripts:**

- The `.session.json` file in each session folder is machine-readable JSON
- You can write scripts to parse this file and create tickets programmatically
- Example use case: nightly script that scans for new sessions and auto-creates draft tickets

---

### Session Archiving and Backup

Because Unbroken QA Capture stores everything as local files, you have full control over archiving and backup.

**Best Practices:**

1. **Regular Backups:** Set up automatic backup of your Sessions Root folder (use Windows Backup, OneDrive, Dropbox, or a dedicated backup tool).

2. **Archive Old Sessions:** After tickets are created and bugs are fixed, you can archive old session folders to free up disk space:
   - Move old sessions to an "Archive" folder or external drive
   - Compress old sessions with Windows zip compression (right-click → Send to → Compressed folder)
   - Delete sessions older than 6 months if you don't need the historical data

3. **Selective Retention:** You don't need to keep every session forever. Consider:
   - Keep sessions with critical bugs or feature requests indefinitely
   - Archive sessions from regular testing after tickets are created
   - Delete test/practice sessions immediately

4. **Cloud Sync (Optional):** If you work from multiple machines or want off-site backup:
   - Set your Sessions Root to a OneDrive, Dropbox, or Google Drive folder
   - All sessions will automatically sync to the cloud
   - Be aware of storage limits (sessions with many screenshots can be several GB)

---

### Summary: Key Points

- **Every QA session creates a timestamped folder** with all evidence organized by bug
- **Each bug gets its own subfolder** with screenshots, notes, and a formatted description
- **`description.md` files are ready for ticketing** — copy to clipboard and paste into Linear, Jira, or any other tool
- **The ticket template is fully customizable** to match your team's conventions
- **All output is portable** — plain files and folders, no proprietary formats, no cloud lock-in
- **Backup is simple** — just back up the Sessions Root folder like any other document folder

This file-based approach means Unbroken QA Capture works with any ticketing workflow, now and in the future.

---

## Settings Reference

Unbroken QA Capture is designed to work well out-of-the-box, but every team and tester has unique preferences. This section documents all available settings, what they do, and recommended values for common scenarios.

Settings are accessible via:
- **System Tray:** Right-click the Unbroken icon → Settings
- **Main Window:** Menu → Settings (when in Review Mode or during setup)

All settings are stored in `%APPDATA%\UnbrokenQACapture\settings.json` on Windows. They persist across sessions and app restarts.

---

### General Settings

These settings control the application's core behavior and file storage.

#### Sessions Root Folder

**What it does:** Sets the directory where all QA session folders are created.

**Default:** `C:\Users\{YourUsername}\Documents\UnbrokenQACapture\`

**To change:**
1. Settings → General → Sessions Root Folder
2. Click **Browse** to select a new location
3. The app will create the folder if it doesn't exist
4. Click **Save** — future sessions will use the new location

**Considerations:**
- Choose a location with sufficient disk space (sessions can grow to several GB if you capture many screenshots or videos)
- If you use cloud sync (OneDrive, Dropbox), you can set this to a synced folder for automatic backup
- If you back up your Documents folder regularly, the default location is safe
- Avoid network drives (slow file operations can cause lag during capture)

**Example scenarios:**
- **Standard setup:** Use the default (`Documents\UnbrokenQACapture\`)
- **Cloud sync:** Set to `C:\Users\{You}\OneDrive\QA Sessions\` for automatic OneDrive backup
- **Large team with shared storage:** Set to a shared network drive (note: this may be slower)
- **Multiple testers on one machine:** Each user has their own `%APPDATA%` settings, so they can each configure their own Sessions Root

---

#### Launch on Windows Startup

**What it does:** Automatically starts Unbroken QA Capture when you log in to Windows.

**Default:** Off (disabled)

**To change:**
1. Settings → General → Launch on Windows Startup
2. Check the box to enable, uncheck to disable
3. Click **Save**

**How it works:**
- When enabled, the app adds itself to the Windows Startup registry key (`HKCU\Software\Microsoft\Windows\CurrentVersion\Run`)
- The app starts minimized to the system tray (no window appears)
- You can start a session immediately with `Ctrl+Shift+Q`

**When to enable:**
- If you do QA testing frequently throughout the day
- If you want the app always available via global hotkeys
- If you forget to launch the app before testing sessions

**When to disable:**
- If you only do QA testing occasionally
- If you want to control exactly when the app runs
- If you're concerned about memory usage (the app uses <30 MB when idle)

---

#### Minimize to Tray on Close

**What it does:** When you close the main window, the app minimizes to the system tray instead of exiting completely.

**Default:** On (enabled)

**To change:**
1. Settings → General → Minimize to Tray on Close
2. Check to enable, uncheck to disable
3. Click **Save**

**Behavior when enabled:**
- Clicking the X button on the main window minimizes the app to the tray
- The app continues running (hotkeys remain active)
- To fully exit, right-click the tray icon → Exit

**Behavior when disabled:**
- Clicking the X button exits the app completely
- All hotkeys stop working
- You must relaunch the app from the Start menu to use it again

**Recommended:** Leave this enabled. Most users expect a tray-based app to minimize to tray rather than exit.

---

### Hotkey Settings

All global hotkeys can be customized to avoid conflicts with other applications or match your personal preferences.

**To customize a hotkey:**
1. Settings → Hotkeys
2. Click on the hotkey you want to change (it will show "Press a key combination...")
3. Press your desired key combination (e.g., `Ctrl+Alt+Q`)
4. If the combination is already registered by another app, you'll see a warning
5. Click **Save** to apply changes

---

#### Start / End Session

**What it does:** Toggles a QA session on/off (press once to start, press again to end and enter Review Mode).

**Default:** `Ctrl+Shift+Q`

**Available in:** All application states (idle, active session, bug capture, review)

**Conflict considerations:**
- Avoid combinations used by applications you frequently test (e.g., Chrome, VS Code, Slack)
- `Ctrl+Shift+Q` is rarely used by other apps, making it a safe default

**Alternative suggestions:**
- `Ctrl+Alt+Q` (if `Ctrl+Shift+Q` conflicts)
- `F9` (if you prefer a single-key shortcut)
- `Ctrl+Shift+S` (mnemonic: "S" for Session)

---

#### Start Bug Capture + Screenshot

**What it does:** Starts a new bug capture (Bug-01, Bug-02, etc.) and immediately triggers the Windows Snipping Tool for screenshot capture.

**Default:** `Print Screen`

**Available in:** Active Session, Bug Capture Mode

**Why Print Screen:**
- It's the most intuitive key for taking screenshots
- Most testers instinctively press Print Screen when they see a bug
- It's rarely used for other purposes in modern workflows (Snipping Tool has replaced the old "clipboard screenshot" behavior)

**Conflict considerations:**
- Some screenshot tools (ShareX, Greenshot, etc.) also use Print Screen
- If you have another screenshot tool running, you may need to disable it or choose a different hotkey here

**Alternative suggestions:**
- `F8` (if Print Screen is taken)
- `Ctrl+Print Screen` (adds a modifier to avoid conflicts)
- `Ctrl+Shift+C` (mnemonic: "C" for Capture)

---

#### End Bug Capture

**What it does:** Ends the current bug capture and returns to Active Session mode. All screenshots, notes, and metadata are saved to the bug's folder.

**Default:** `F4`

**Available in:** Bug Capture Mode only

**Why F4:**
- It's easy to reach (no modifier keys required)
- It's rarely used by other applications (unlike F1, F5, F11, F12 which are common)
- It's distinct from the Print Screen capture key

**Alternative suggestions:**
- `F5`, `F6`, `F7` (other F-keys that are rarely used)
- `Ctrl+Shift+E` (mnemonic: "E" for End)
- `Escape` (though this might conflict with "cancel" actions in some apps)

---

#### Toggle Quick Notepad (Current Bug)

**What it does:** Opens or closes the quick notepad for the current bug. Use this to paste meeting URLs, jot context notes, or add any text information related to the bug.

**Default:** `Ctrl+Shift+N`

**Available in:** Bug Capture Mode only

**Conflict considerations:**
- Many browsers and apps use `Ctrl+Shift+N` for "New Private/Incognito Window"
- If you frequently test browsers, consider changing this hotkey

**Alternative suggestions:**
- `Ctrl+Shift+J` (mnemonic: "J" for Jot notes)
- `F7` (if you prefer an F-key)
- `Ctrl+Alt+N` (different modifier to avoid browser conflict)

---

#### Toggle Session Notepad

**What it does:** Opens or closes the session notepad. Use this for session-wide notes that aren't specific to a single bug (e.g., test environment details, general observations).

**Default:** `Ctrl+Shift+M`

**Available in:** Active Session, Bug Capture Mode

**Conflict considerations:**
- `Ctrl+Shift+M` is used by some developer tools (e.g., Chrome DevTools device toggle)
- If you frequently test web apps with DevTools open, consider changing this

**Alternative suggestions:**
- `Ctrl+Shift+S` (mnemonic: "S" for Session notes — if not used for Start Session)
- `F6` (if you prefer an F-key)
- `Ctrl+Alt+M` (different modifier)

---

#### Hotkey Conflict Detection

When you set a hotkey, Unbroken QA Capture checks if another application has already registered that combination globally.

**Green checkmark:** The hotkey is available and will work.

**Warning icon:** Another application has registered this hotkey. You can:
- **Override:** Use it anyway (the other app's hotkey will stop working)
- **Choose a different combination:** Click "Cancel" and try another key combination

**Note:** Windows doesn't always report which application owns a conflicting hotkey. If you see a conflict but don't know which app is using it, try closing applications one at a time until the conflict clears.

---

### Annotation Settings

These settings control the behavior and defaults of the annotation tool.

#### Auto-Open Annotation Tool on Screenshot

**What it does:** Automatically opens the annotation window immediately after you capture a screenshot.

**Default:** Off (disabled)

**To change:**
1. Settings → Annotation → Auto-Open on Screenshot
2. Check to enable, uncheck to disable
3. Click **Save**

**Behavior when enabled:**
- After the Windows Snipping Tool saves a screenshot, the annotation window opens automatically
- You can immediately start annotating without manually opening the tool
- Press `Escape` to skip annotation if you don't need it for this screenshot

**Behavior when disabled:**
- Screenshots are saved without opening the annotation tool
- You can annotate later during Review Mode (right-click a screenshot → Annotate)

**When to enable:**
- If you annotate most or all of your screenshots
- If you want to add text labels or arrows while the bug is fresh in your mind

**When to disable:**
- If you only annotate some screenshots (most are self-explanatory)
- If you want a faster capture flow during active testing and prefer to annotate later

---

#### Default Annotation Save Mode

**What it does:** Controls what happens when you save annotations.

**Default:** "Create new file" (saves as `capture-NNN_annotated.png`, leaving the original unchanged)

**Options:**
- **Create new file:** Saves annotated version with `_annotated` suffix, keeps original
- **Overwrite original:** Replaces the original screenshot with the annotated version
- **Ask each time:** Prompts you to choose when you press `Ctrl+S`

**To change:**
1. Settings → Annotation → Save Mode
2. Select your preferred option
3. Click **Save**

**Recommended:** Use "Create new file" (default). This preserves the original screenshot in case you need to re-annotate or show both versions (annotated for the ticket, original for detailed analysis).

---

#### Default Annotation Color

**What it does:** Sets the default color for text labels, rectangles, ovals, and freehand drawing when you open the annotation tool.

**Default:** Red (`#FF3B30`)

**To change:**
1. Settings → Annotation → Default Color
2. Click the color picker and choose a color
3. Click **Save**

**Common choices:**
- **Red:** High contrast, draws attention to errors (good for bug annotations)
- **Yellow:** Softer, good for highlighting areas without implying "wrong"
- **Blue:** Professional, good for informational annotations
- **Green:** Positive tone, good for highlighting expected behavior

**Note:** You can always change the color while annotating using the color picker in the annotation toolbar. This setting just controls the initial default.

---

#### Default Stroke Width

**What it does:** Sets the default line thickness for rectangles, ovals, and freehand drawing.

**Default:** 3 pixels

**Range:** 1-10 pixels

**To change:**
1. Settings → Annotation → Default Stroke Width
2. Use the slider to adjust (1 = thin, 10 = very thick)
3. Click **Save**

**Recommended values:**
- **1-2px:** Fine details, small UI elements, precise annotations
- **3-4px:** Good balance for most use cases (default)
- **5-10px:** Bold annotations for presentations or high-level overviews

---

### AI Settings

These settings control Claude AI integration and behavior.

#### Claude CLI Detection Status

**What it shows:** Whether Claude Code CLI is installed and authenticated on your system.

**Possible states:**
- **Green checkmark:** Claude is installed, authenticated, and ready
- **Yellow warning:** Claude is installed but not authenticated (run `claude` in terminal to log in)
- **Red X:** Claude is not installed (install with `npm install -g @anthropic-ai/claude-code`)

**Actions:**
- **Recheck:** Manually recheck Claude CLI status (useful after installing or authenticating)
- **Disable AI Features:** Turn off all AI functionality (the app works as a manual capture tool)

---

#### Enable AI for Description Generation

**What it does:** Controls whether the "Generate Description" button appears in Review Mode.

**Default:** On (enabled, if Claude is available)

**To change:**
1. Settings → AI → Enable AI for Description Generation
2. Check to enable, uncheck to disable
3. Click **Save**

**When to disable:**
- If you prefer to write all bug descriptions manually
- If you're concerned about Claude API costs (though Claude Code CLI uses your existing subscription)
- If Claude is producing low-quality descriptions for your specific use case

---

#### Enable AI for Console Parsing

**What it does:** Controls whether Claude attempts to parse console screenshots for error messages and stack traces.

**Default:** On (enabled, if Claude is available)

**To change:**
1. Settings → AI → Enable AI for Console Parsing
2. Check to enable, uncheck to disable
3. Click **Save**

**When to disable:**
- If your console screenshots rarely contain useful error messages
- If you prefer to manually copy-paste console output instead

---

#### Enable AI for Session Summaries

**What it does:** Controls whether the "Generate Session Summary" button appears in Review Mode.

**Default:** On (enabled, if Claude is available)

**To change:**
1. Settings → AI → Enable AI for Session Summaries
2. Check to enable, uncheck to disable
3. Click **Save**

**When to disable:**
- If you don't need session-level summaries
- If you prefer to write summaries manually

---

#### Cost Tracking Display

**What it shows:** Estimated Claude API cost for AI operations in the current session.

**Note:** Claude Code CLI uses your existing Claude subscription (Pro or Team). There's no per-token charge beyond your subscription cost. This display shows estimated token usage, not actual billing.

**To reset:** Cost tracking resets when you start a new session.

---

### Ticketing Settings

These settings control how bug descriptions are formatted for your ticketing system.

#### Integration Type

**What it does:** Selects which ticketing system format to use.

**Default:** Linear (file-based)

**Options in v1:**
- **Linear (file-based):** Formats output for Linear using markdown (copy-paste workflow)
- **None:** No ticketing integration (just use the organized folders and plain markdown files)

**Future versions** will add:
- **Linear (API):** Direct integration with Linear GraphQL API (auto-create tickets)
- **Jira:** Format for Jira's markdown syntax
- **GitHub Issues:** Format for GitHub's issue markdown
- **Custom:** User-defined template

---

#### Ticket Template Editor

**What it does:** Opens the ticket template file for editing.

**To customize:**
1. Settings → Ticketing → Edit Ticket Template
2. The template file opens in your default markdown editor
3. Modify the structure, field names, or formatting
4. Save the file — changes take effect immediately

**To reset to default:**
1. Settings → Ticketing → Reset to Default Template
2. Confirm the action
3. The default template is restored

See the "Output & Ticketing" section above for template variable reference and examples.

---

#### Default Bug Type

**What it does:** Sets the default type for new bugs (Bug, Feature, or Feedback).

**Default:** Bug

**Options:**
- **Bug:** Something is broken or incorrect
- **Feature:** A request for new functionality
- **Feedback:** General UX observations or suggestions

**To change:**
1. Settings → Ticketing → Default Bug Type
2. Select from the dropdown
3. Click **Save**

**Note:** You can override this for individual bugs during Review Mode. This just sets the default to reduce repetitive selection.

---

### About

The About section displays:
- **App Version:** Current installed version (e.g., `1.0.0`)
- **Unbroken Technology Branding:** Company logo and tagline
- **Links:**
  - Website: https://unbrokentechnology.com
  - Support: https://unbrokentechnology.com/support
  - Changelog: https://unbrokentechnology.com/qa-capture/changelog
  - License: View the software license

---

### Advanced Settings

These settings are for troubleshooting and debugging. Most users won't need to change these.

#### Enable Debug Logging

**What it does:** Writes detailed logs to `%APPDATA%\UnbrokenQACapture\logs\` for troubleshooting.

**Default:** Off (disabled)

**To enable:**
1. Settings → Advanced → Enable Debug Logging
2. Check the box
3. Click **Save**

**When to enable:**
- If you're experiencing crashes or errors
- If hotkeys aren't working as expected
- If the file watcher isn't detecting screenshots
- If submitting a bug report to Unbroken Technology support

**Note:** Debug logs can grow large over time. Disable this after troubleshooting is complete.

---

#### View Logs

**What it does:** Opens the logs folder in File Explorer.

**To access:**
1. Settings → Advanced → View Logs
2. The folder `%APPDATA%\UnbrokenQACapture\logs\` opens
3. Look for files named by date (e.g., `2026-02-16.log`)

---

#### Reset All Settings to Default

**What it does:** Restores all settings to their default values (as if running the first-run wizard again).

**To reset:**
1. Settings → Advanced → Reset All Settings
2. Confirm the action (this cannot be undone)
3. The app restarts with default settings

**Warning:** This does NOT delete your session data. Only settings are reset.

---

### Settings File Location

All settings are stored in JSON format at:

**Windows:** `%APPDATA%\UnbrokenQACapture\settings.json`

You can manually edit this file if needed (close the app first, or changes will be overwritten). The file structure is:

```json
{
  "sessionsRoot": "C:\\Users\\YourName\\Documents\\UnbrokenQACapture",
  "launchOnStartup": false,
  "minimizeToTray": true,
  "hotkeys": {
    "startEndSession": "Ctrl+Shift+Q",
    "startBugCapture": "PrintScreen",
    "endBugCapture": "F4",
    "toggleQuickNotepad": "Ctrl+Shift+N",
    "toggleSessionNotepad": "Ctrl+Shift+M"
  },
  "annotation": {
    "autoOpen": false,
    "saveMode": "createNew",
    "defaultColor": "#FF3B30",
    "defaultStrokeWidth": 3
  },
  "ai": {
    "enableDescriptionGeneration": true,
    "enableConsoleParsing": true,
    "enableSessionSummary": true
  },
  "ticketing": {
    "integrationType": "linear-file",
    "defaultBugType": "bug"
  },
  "advanced": {
    "debugLogging": false
  }
}
```

**Backup tip:** Copy this file before making manual edits. If you break the JSON syntax, the app will reset to defaults on next launch.

---

## Troubleshooting

This section covers common issues you might encounter with Unbroken QA Capture and how to resolve them.

---

### Installation and Startup Issues

#### Problem: "Windows SmartScreen prevented an unrecognized app from starting"

**Cause:** Windows SmartScreen blocks apps from unknown publishers by default.

**Solution:**
1. Click **"More info"** in the SmartScreen dialog
2. Click **"Run anyway"**
3. The app will install normally

**Why this happens:** Unbroken Technology is not yet a widely recognized publisher. Once the app has enough installations and positive reputation, SmartScreen warnings will stop appearing.

---

#### Problem: App won't launch after installation

**Symptoms:**
- Double-clicking the app icon does nothing
- No error message appears
- The tray icon doesn't appear

**Solution 1: Check Windows Event Viewer**
1. Open Windows Event Viewer (search "Event Viewer" in Start menu)
2. Navigate to Windows Logs → Application
3. Look for errors from "UnbrokenQACapture" or "Tauri"
4. Note the error message and see specific error sections below

**Solution 2: Run as Administrator (temporarily)**
1. Right-click the Unbroken QA Capture shortcut
2. Select "Run as administrator"
3. If the app launches, the issue is permissions-related (see next solution)

**Solution 3: Check %APPDATA% permissions**
1. Open File Explorer
2. Navigate to `C:\Users\{YourUsername}\AppData\Roaming\`
3. Right-click → Properties → Security tab
4. Ensure your user account has "Full control" permissions
5. If not, click Edit, select your account, and grant Full control

---

#### Problem: App crashes immediately on startup

**Symptoms:**
- App launches briefly then disappears
- Tray icon appears then vanishes
- No visible error message

**Solution 1: Delete corrupted settings file**
1. Navigate to `%APPDATA%\UnbrokenQACapture\`
2. Rename `settings.json` to `settings.json.backup`
3. Launch the app again (it will run the first-run wizard)
4. If this fixes the issue, your settings file was corrupted

**Solution 2: Delete corrupted database**
1. Navigate to `%APPDATA%\UnbrokenQACapture\`
2. Rename `bugcatcher.db` to `bugcatcher.db.backup`
3. Launch the app again (it will create a new database)
4. If this fixes the issue, your database was corrupted

**Solution 3: Enable debug logging and check logs**
1. Manually create a settings file with debug logging enabled:
   - Create `%APPDATA%\UnbrokenQACapture\settings.json`
   - Add: `{"advanced": {"debugLogging": true}}`
2. Launch the app
3. Check `%APPDATA%\UnbrokenQACapture\logs\` for error details

---

### Claude AI Issues

#### Problem: "Claude CLI not found" error

**Symptoms:**
- First-run wizard shows "Claude is not installed"
- Settings → AI shows red X for Claude status
- "Generate Description" button is grayed out

**Solution 1: Install Claude Code CLI**
1. Open a terminal (Windows Terminal, PowerShell, or Command Prompt)
2. Run: `npm install -g @anthropic-ai/claude-code`
3. Wait for installation to complete
4. Return to Unbroken QA Capture → Settings → AI → Recheck

**If npm is not installed:**
1. Download Node.js from https://nodejs.org/ (LTS version recommended)
2. Run the installer (default options are fine)
3. Close and reopen your terminal
4. Verify installation: `node --version` and `npm --version`
5. Then install Claude CLI (see Solution 1 above)

**Solution 2: Check PATH environment variable**

If you installed Claude CLI but the app still can't find it:
1. Open a terminal
2. Run: `where claude` (on Windows) or `which claude` (on macOS)
3. If this returns a path, Claude is installed but not in the app's PATH
4. Restart Unbroken QA Capture (it reads PATH on startup)
5. If still not working, add the Claude installation directory to your system PATH:
   - Open System Properties → Environment Variables
   - Edit the "Path" variable under "User variables"
   - Add the directory where `claude.exe` is located (usually `%APPDATA%\npm\`)
   - Click OK and restart the app

---

#### Problem: "Claude is not authenticated" warning

**Symptoms:**
- Settings → AI shows yellow warning for Claude status
- "Generate Description" button doesn't work
- Error message: "Claude CLI is not authenticated"

**Solution:**
1. Open a terminal (Windows Terminal, PowerShell, or Command Prompt)
2. Run: `claude`
3. A browser window will open asking you to log in to Claude.ai
4. Log in with your Claude account (you need a Pro or Team subscription)
5. After successful login, the terminal will show "Authentication successful"
6. Return to Unbroken QA Capture → Settings → AI → Recheck
7. You should see a green checkmark: "Claude AI is ready"

**If you don't have a Claude subscription:**
- Claude AI features require an active Claude Pro or Claude Team subscription
- Visit https://claude.ai/settings/billing to subscribe
- Free tier is not supported for Claude Code CLI

---

#### Problem: "Generate Description" hangs or times out

**Symptoms:**
- Clicking "Generate Description" shows a loading spinner that never completes
- After 30 seconds, you see "Request timed out"
- The description is not generated

**Solution 1: Check Claude CLI manually**
1. Open a terminal
2. Run: `claude --print "Hello" --output-format json`
3. You should see a JSON response within a few seconds
4. If this hangs or errors, the issue is with Claude CLI, not Unbroken QA Capture

**Solution 2: Check your internet connection**
- Claude CLI requires internet access to reach Anthropic's API
- Verify you can access https://claude.ai in a browser
- Check if you're behind a corporate firewall or proxy that blocks API calls

**Solution 3: Reduce screenshot count**
- Claude's vision processing can take longer with many screenshots
- Try generating a description for a bug with only 1-2 screenshots
- If this works, the timeout was due to processing time, not a configuration issue

**Solution 4: Check Claude API status**
- Visit https://status.anthropic.com
- If there's an outage, you'll see it reported there
- Wait for Anthropic to resolve the issue, then try again

---

#### Problem: AI descriptions are low-quality or generic

**Symptoms:**
- Claude generates descriptions like "There is an issue visible in the screenshot"
- Descriptions don't include specific details from your screenshots
- Console errors aren't parsed correctly

**Solution 1: Add more context in notes**
- Claude generates better descriptions when you provide context
- Use the quick notepad (`Ctrl+Shift+N`) to add:
  - What you were doing when the bug occurred
  - What you expected to happen
  - Any error messages or specific UI text visible in screenshots
- Then click "Generate Description" — Claude will incorporate your notes

**Solution 2: Use the refinement feature**
- After Claude generates a description, type a refinement instruction:
  - "Add more detail about the console error"
  - "Explain the steps to reproduce more clearly"
  - "Focus on the UI element highlighted in red"
- Click "Refine Description" — Claude will revise the description

**Solution 3: Annotate screenshots before generating**
- Use the annotation tool to add text labels and arrows
- Highlight the specific UI elements or error messages that matter
- Claude can see annotations and will reference them in descriptions

**Solution 4: Manually edit the description**
- Claude's output is a starting point, not a final product
- Click "Edit" and manually improve the description
- Future refinements will use your edits as a base

---

### Hotkey Issues

#### Problem: Hotkeys don't work at all

**Symptoms:**
- Pressing `Ctrl+Shift+Q` does nothing
- Print Screen doesn't start a bug capture
- No response from any global hotkeys

**Solution 1: Check if the app is running**
- Look for the Unbroken QA Capture icon in your system tray (notification area)
- If it's not there, launch the app from the Start menu
- Global hotkeys only work when the app is running

**Solution 2: Check for hotkey conflicts**
1. Settings → Hotkeys
2. Look for warning icons next to hotkeys
3. If you see a conflict, another app has registered that hotkey
4. Change the hotkey to a different combination

**Solution 3: Restart the app**
- Sometimes Windows fails to register global hotkeys on startup
- Right-click the tray icon → Exit
- Relaunch the app from the Start menu
- Hotkeys should work after restart

**Solution 4: Check Windows hotkey limits**
- Windows has a system-wide limit on registered global hotkeys (~100-150)
- If you run many apps with global hotkeys, some may fail to register
- Close other apps with global hotkeys and restart Unbroken QA Capture

---

#### Problem: Print Screen opens Snipping Tool but doesn't start a bug capture

**Symptoms:**
- Pressing Print Screen opens the Windows Snipping Tool
- You capture a screenshot, but it's not associated with a bug
- The screenshot isn't moved to the session folder

**Solution 1: Ensure you're in an active session**
- Print Screen only works during an active session
- Press `Ctrl+Shift+Q` to start a session first
- The tray icon should turn green (indicating active session)
- Then press Print Screen to start a bug capture

**Solution 2: Check registry redirect**
- The app redirects Snipping Tool output to the session folder via Windows Registry
- If the registry key wasn't set correctly, screenshots go to the default location
- Settings → Advanced → View Logs → look for "Registry redirect failed" errors
- If you see this error, try running the app as administrator once (right-click → Run as administrator)

**Solution 3: Check file watcher**
- The file watcher monitors the `_captures/` folder for new screenshots
- If the watcher isn't running, screenshots won't be auto-sorted
- Enable debug logging (Settings → Advanced → Enable Debug Logging)
- Take a screenshot and check the logs for file watcher events
- If you see "File watcher not started" errors, restart the app

---

#### Problem: Hotkey conflict with another application

**Symptoms:**
- You press a hotkey and the wrong application responds
- Settings shows a warning icon for a specific hotkey
- Error message: "Hotkey already registered by another application"

**Solution 1: Change the hotkey in Unbroken QA Capture**
1. Settings → Hotkeys
2. Click the conflicting hotkey
3. Press a different key combination
4. Click Save

**Solution 2: Identify and close the conflicting app**
1. Note which hotkey is conflicting (e.g., `Ctrl+Shift+Q`)
2. Close applications one by one
3. After closing each app, Settings → Hotkeys → click the hotkey to re-test
4. When the conflict clears, you've found the conflicting app
5. Either keep that app closed during QA sessions or change the hotkey

**Common conflicting apps:**
- **Print Screen:** ShareX, Greenshot, Lightshot, Snagit
- **Ctrl+Shift+N:** Chrome, Edge, Firefox (new private window)
- **F-keys:** Certain games or game launchers (F4, F5, F12)

---

### Screenshot and Capture Issues

#### Problem: Screenshots aren't being captured

**Symptoms:**
- Pressing Print Screen opens Snipping Tool, but the screenshot isn't saved to the session folder
- No files appear in the bug's `screenshots/` folder
- The file watcher doesn't detect screenshots

**Solution 1: Check Snipping Tool save location**
1. After taking a screenshot with Snipping Tool, check where it's saving
2. It should save to: `{Sessions Root}/{Session Folder}/_captures/`
3. If it's saving to the default location (Pictures\Screenshots), the registry redirect failed

**Solution 2: Manually trigger registry redirect**
1. End any active session (press `Ctrl+Shift+Q` if in a session)
2. Start a new session (`Ctrl+Shift+Q`)
3. Check Settings → Advanced → View Logs for "Registry redirect successful" message
4. If you see "Registry redirect failed", try running as administrator

**Solution 3: Use manual screenshot workflow (workaround)**
If the automatic redirect isn't working:
1. Take screenshots with Snipping Tool as normal (they save to default location)
2. Manually drag screenshots from Pictures\Screenshots to the bug folder in File Explorer
3. Submit a support ticket to Unbroken Technology with your debug logs

---

#### Problem: Registry not restored after session ends

**Symptoms:**
- After ending a session, Snipping Tool still saves to the old session folder
- Screenshots from outside a session go to the wrong location
- Registry key `HKCU\Software\Microsoft\Windows\TabletPC\Snipping Tool\LastUsedSaveAsLocation` has the wrong value

**Solution 1: Manually end the session properly**
- Ensure you press `Ctrl+Shift+Q` to end the session (don't just close the app)
- The registry restore happens during the "end session" process
- If you force-close the app (Task Manager), the registry may not be restored

**Solution 2: Manually restore the registry**
1. Open Registry Editor (search "regedit" in Start menu)
2. Navigate to `HKEY_CURRENT_USER\Software\Microsoft\Windows\TabletPC\Snipping Tool\`
3. Find the key `LastUsedSaveAsLocation`
4. Set it to your desired default location (e.g., `C:\Users\{You}\Pictures\Screenshots`)
5. Close Registry Editor

**Solution 3: Check for cached registry value**
- The app caches the original registry value when you start a session
- If the cache file is corrupted, the restore may fail
- Navigate to `%APPDATA%\UnbrokenQACapture\`
- Delete any files named `registry-cache-*.json`
- Restart the app and start a new session

**Prevention:**
- Always end sessions properly with `Ctrl+Shift+Q` (don't force-close the app)
- If the app crashes during a session, the registry may not be restored — manually restore it per Solution 2 above

---

#### Problem: Annotation window won't open

**Symptoms:**
- Right-clicking a screenshot → Annotate does nothing
- No annotation window appears
- No error message

**Solution 1: Check if the window is hidden**
- Press `Alt+Tab` to see all open windows
- The annotation window may be behind another window
- Click it to bring it to the front

**Solution 2: Check display settings (multi-monitor setups)**
- If you have multiple monitors, the annotation window may open on a different screen
- Try disconnecting secondary monitors temporarily
- Reopen the annotation window — it should appear on your primary monitor
- Reconnect monitors and the app will remember the correct display

**Solution 3: Reset window position**
1. Navigate to `%APPDATA%\UnbrokenQACapture\settings.json`
2. Open in a text editor
3. Find and delete the `"annotationWindowPosition"` entry
4. Save the file and restart the app
5. The annotation window will reset to the center of your primary monitor

---

### Session and File Issues

#### Problem: Session folder is empty or missing bugs

**Symptoms:**
- You completed a session with multiple bugs, but the session folder only has a few bugs or none
- Bug folders are created but contain no screenshots
- The `_captures/` folder has screenshots that weren't sorted

**Solution 1: Wait for file watcher to finish**
- The file watcher may take a few seconds to process screenshots, especially with many files
- Wait 5-10 seconds after ending the session before checking the folder
- Refresh File Explorer (press F5) to see newly sorted files

**Solution 2: Check file watcher logs**
1. Enable debug logging (Settings → Advanced → Enable Debug Logging)
2. Start a session, take a screenshot, and wait
3. Check logs for errors like "File watcher failed to move file"
4. Common causes: file permissions, antivirus blocking file moves, disk full

**Solution 3: Manually sort files from `_captures/`**
If the file watcher failed:
1. Open the session folder in File Explorer
2. Navigate to `_captures/`
3. Manually drag files into the appropriate `Bug-XX/screenshots/` folders
4. Submit a support ticket with your debug logs

---

#### Problem: Session summary or descriptions are missing

**Symptoms:**
- After clicking "Generate Session Summary", no `session-summary.md` file is created
- Bug folders don't have `description.md` files even after generating descriptions

**Solution 1: Check if AI features are enabled**
1. Settings → AI
2. Ensure "Enable AI for Description Generation" and "Enable AI for Session Summaries" are checked
3. If unchecked, enable them and try again

**Solution 2: Check Claude CLI status**
- These files are generated by Claude AI
- If Claude is not installed or authenticated, descriptions can't be generated
- See "Claude AI Issues" section above for troubleshooting

**Solution 3: Check file permissions**
- The app may not have permission to write to the session folder
- Right-click the session folder → Properties → Security
- Ensure your user account has "Modify" permissions

---

#### Problem: Can't find old sessions

**Symptoms:**
- You completed a session, but can't find the folder later
- The Sessions Root folder seems empty or only has recent sessions

**Solution 1: Check Sessions Root location**
1. Settings → General → Sessions Root Folder
2. Note the path (e.g., `C:\Users\{You}\Documents\UnbrokenQACapture\`)
3. Open File Explorer and navigate to that exact path
4. Look for folders named `YYYY-MM-DD_{ID}` (e.g., `2026-02-16_a3f2`)

**Solution 2: Search by date**
- In File Explorer, navigate to the Sessions Root
- Use the search box (top-right) to search for session dates
- Example: search for `2026-02-15` to find all sessions from February 15

**Solution 3: Check .session.json files**
- Each session folder contains a `.session.json` file with metadata
- Use Windows Search to find all `.session.json` files in your Documents folder
- This will show you every session folder, even if they were moved

---

### Performance Issues

#### Problem: App is slow or unresponsive during review

**Symptoms:**
- Review Mode takes a long time to load
- Scrolling through bug cards is laggy
- Clicking on bugs has a 2-3 second delay

**Solution 1: Reduce thumbnail quality**
- The app generates thumbnails for all screenshots in review
- With 30+ bugs and many screenshots each, this can be memory-intensive
- Workaround: close and reopen Review Mode to clear memory
- Future versions will add a setting to disable thumbnails

**Solution 2: Review bugs in batches**
- Instead of reviewing 30 bugs at once, review 5-10 at a time
- Mark bugs as "reviewed" as you go
- Filter the review list to show only "not reviewed" bugs

**Solution 3: Close other applications**
- Review Mode uses ~150 MB of RAM with many thumbnails loaded
- Close other memory-intensive applications (Chrome, Slack, etc.) during review

---

#### Problem: High RAM usage when idle

**Symptoms:**
- Task Manager shows Unbroken QA Capture using 100+ MB of RAM when idle (no active session)
- Expected: <30 MB when idle

**Solution 1: Check for memory leaks**
1. Right-click the tray icon → Exit
2. Relaunch the app from the Start menu
3. Check RAM usage in Task Manager
4. If it's still high after a fresh launch, submit a bug report with debug logs

**Solution 2: Check for file watcher leaks**
- If you didn't properly end a session (e.g., force-closed the app), the file watcher may still be running
- Restart the app (this stops all background watchers)

---

### Other Issues

#### Problem: Tray icon doesn't appear

**Symptoms:**
- The app is running (you can see it in Task Manager), but no tray icon
- You can't access the app menu or settings

**Solution 1: Check system tray overflow**
1. Click the "^" icon in your system tray (near the clock)
2. Look for the Unbroken QA Capture icon in the overflow menu
3. Right-click the Windows taskbar → Taskbar settings → Select which icons appear on the taskbar
4. Find "Unbroken QA Capture" and toggle it to "On"

**Solution 2: Restart Windows Explorer**
1. Open Task Manager (`Ctrl+Shift+Esc`)
2. Find "Windows Explorer" in the process list
3. Right-click → Restart
4. The tray icon should reappear

**Solution 3: Reinstall the app**
- If the tray icon never appears even after a restart, the installation may be corrupted
- Uninstall Unbroken QA Capture via Settings → Apps
- Download a fresh installer and reinstall

---

#### Problem: Settings changes don't persist

**Symptoms:**
- You change a setting, click Save, but after restarting the app the old value is back
- Settings panel always shows default values

**Solution 1: Check file permissions**
1. Navigate to `%APPDATA%\UnbrokenQACapture\`
2. Right-click `settings.json` → Properties → Security
3. Ensure your user account has "Modify" permissions
4. If not, grant permissions and try changing settings again

**Solution 2: Check for read-only flag**
1. Navigate to `%APPDATA%\UnbrokenQACapture\`
2. Right-click `settings.json` → Properties
3. Uncheck "Read-only" if it's checked
4. Click OK and try changing settings again

**Solution 3: Manually edit settings.json**
1. Close Unbroken QA Capture (Exit from tray icon)
2. Navigate to `%APPDATA%\UnbrokenQACapture\`
3. Open `settings.json` in Notepad or another text editor
4. Make your changes (ensure valid JSON syntax)
5. Save the file
6. Relaunch the app

---

### Getting Help

If you've tried the troubleshooting steps above and still have issues:

**1. Collect Debug Logs**
- Settings → Advanced → Enable Debug Logging
- Reproduce the issue
- Settings → Advanced → View Logs → copy the latest log file

**2. Check the FAQ**
- Visit https://unbrokentechnology.com/qa-capture/faq
- Search for your issue — many common problems have quick solutions

**3. Submit a Support Ticket**
- Visit https://unbrokentechnology.com/support
- Include:
  - Your Windows version (Settings → System → About)
  - Unbroken QA Capture version (Settings → About)
  - A description of the issue and steps to reproduce
  - Your debug log file (if applicable)

**4. Community Forum**
- Visit https://unbrokentechnology.com/community
- Search for similar issues or ask a question
- Other users and the Unbroken team can help

---

## Keyboard Shortcuts Reference

Unbroken QA Capture uses global hotkeys to keep the tool accessible without interrupting your testing workflow. This section provides a complete reference of all keyboard shortcuts available in the application.

### Global Hotkeys

These hotkeys work system-wide, even when the application is not in focus. They allow you to control QA sessions and bug captures while actively using the application under test.

| Hotkey | Action | Available In | Description |
|--------|--------|--------------|-------------|
| **Ctrl+Shift+Q** | Start / End Session | All states | Toggle a QA session on/off. Press once to start a session, press again to end it and enter Review Mode. |
| **Print Screen** | Start Bug Capture + Screenshot | Active Session, Bug Capture | Starts a new bug capture (Bug-01, Bug-02, etc.) and triggers the Windows Snipping Tool for screenshot capture. If already in bug capture mode, adds another screenshot to the current bug. |
| **F4** | End Bug Capture | Bug Capture Mode | Ends the current bug capture and returns to Active Session mode. All screenshots, notes, and metadata are saved to the bug's folder. |
| **Ctrl+Shift+N** | Toggle Quick Notepad | Bug Capture Mode | Opens or closes the quick notepad for the current bug. Use this to paste meeting URLs, jot context notes, or add any text information related to the bug. The notepad content is saved with the bug. |
| **Ctrl+Shift+M** | Toggle Session Notepad | Active Session, Bug Capture | Opens or closes the session notepad. Use this for session-wide notes that aren't specific to a single bug (e.g., test environment details, general observations). |

**Notes:**
- All global hotkeys can be customized in Settings → Hotkeys
- If a hotkey conflicts with another application, you'll receive a notification and can choose an alternative in Settings
- Global hotkeys respond within 100ms of the keypress
- The application must be running (system tray icon visible) for global hotkeys to work

---

### Annotation Window Hotkeys

These keyboard shortcuts are available when the annotation window is open. They provide quick access to annotation tools and actions without needing to click toolbar buttons.

#### Tool Selection

| Hotkey | Tool | Description |
|--------|------|-------------|
| **T** | Text Tool | Add text labels and annotations to screenshots. Click to place text, then type your annotation. |
| **R** | Rectangle Tool | Draw rectangular outlines around UI elements. Click and drag to create a rectangle. |
| **O** | Oval Tool | Draw oval/circular outlines around UI elements. Click and drag to create an oval. |
| **D** | Freehand Draw Tool | Draw freehand lines and arrows. Click and drag to draw freely. |

**Tips:**
- Press the tool hotkey again to deselect the current tool and return to selection mode
- Tool hotkeys are single letters (no modifier keys) for fastest access during annotation work
- The active tool is highlighted in the toolbar

#### Editing Actions

| Hotkey | Action | Description |
|--------|--------|-------------|
| **Ctrl+Z** | Undo | Undo the last annotation action. Supports multiple levels of undo. |
| **Ctrl+Shift+Z** | Redo | Redo a previously undone action. |
| **Delete** or **Backspace** | Delete Selected | Delete the currently selected annotation object. |

#### File Operations

| Hotkey | Action | Description |
|--------|--------|-------------|
| **Ctrl+S** | Save and Close | Save all annotations to the screenshot and close the annotation window. The annotated image is saved to the current bug's folder. |
| **Escape** | Cancel and Close | Close the annotation window without saving changes. You'll be prompted to confirm if you have unsaved annotations. |

**Notes:**
- Annotation window hotkeys only work when the annotation window has focus
- Changes are auto-saved to a temporary file as you work (recovery available if the window crashes)
- The annotation window is modal and always-on-top to prevent losing it while working

---

### Application States and Hotkey Availability

Understanding when hotkeys are available helps you work efficiently. The application has four main states:

1. **Idle (Not in Session)**
   - Available hotkeys: `Ctrl+Shift+Q` (to start a session)
   - All other hotkeys are disabled

2. **Active Session**
   - Available hotkeys: `Ctrl+Shift+Q` (to end session), `Print Screen` (to start bug capture), `Ctrl+Shift+M` (for session notepad)
   - The tray icon shows green to indicate an active session

3. **Bug Capture Mode**
   - Available hotkeys: All global hotkeys work in this state
   - `Print Screen` adds more screenshots to the current bug
   - `F4` ends the current bug capture
   - `Ctrl+Shift+N` opens the quick notepad for this bug
   - `Ctrl+Shift+M` opens the session notepad
   - The tray icon shows red to indicate bug capture mode

4. **Review Mode**
   - Available hotkeys: Standard UI navigation (Tab, Enter, Escape)
   - Global hotkeys are disabled during review
   - Review mode is entered automatically when you end a session

---

### Customizing Hotkeys

All global hotkeys can be customized to match your preferences and avoid conflicts with other applications.

**To customize a hotkey:**

1. Open the Settings panel (right-click the tray icon → Settings, or use the main window menu)
2. Navigate to **Settings → Hotkeys**
3. Click on the hotkey you want to change
4. Press your desired key combination
5. The application will check for conflicts:
   - **Green checkmark:** The hotkey is available
   - **Warning icon:** Another application has registered this hotkey (you can choose to override or select a different combination)
6. Click **Save** to apply changes

**Best practices for custom hotkeys:**
- Avoid combinations used by applications you frequently test (e.g., VS Code's `Ctrl+Shift+P`, Chrome's `Ctrl+Shift+T`)
- Keep `Print Screen` as the bug capture trigger if possible — it's intuitive for QA testers
- Choose `F`-keys (`F4`, `F5`, `F6`) for actions that need quick access, as they're rarely used by applications
- Use `Ctrl+Shift+` combinations for session controls to avoid conflicts with standard `Ctrl+` shortcuts

**Hotkey conflicts:**
- If you see a warning about a conflict, the other application has priority
- You can override the conflict, but the other application's hotkey will stop working
- The Settings panel shows which application registered the conflicting hotkey (when Windows provides this information)

---

### Quick Reference Card

Print or save this quick reference for easy access during testing sessions:

```
┌─────────────────────────────────────────────────────────────┐
│          UNBROKEN QA CAPTURE — HOTKEY REFERENCE             │
├─────────────────────────────────────────────────────────────┤
│ SESSION CONTROL                                             │
│   Ctrl+Shift+Q .......... Start / End Session               │
│                                                              │
│ BUG CAPTURE                                                 │
│   Print Screen .......... Start Bug + Screenshot            │
│   F4 .................... End Bug Capture                   │
│                                                              │
│ NOTEPADS                                                    │
│   Ctrl+Shift+N .......... Quick Notepad (current bug)       │
│   Ctrl+Shift+M .......... Session Notepad (all bugs)        │
│                                                              │
│ ANNOTATION TOOLS                                            │
│   T ..................... Text Tool                         │
│   R ..................... Rectangle Tool                    │
│   O ..................... Oval Tool                         │
│   D ..................... Freehand Draw Tool                │
│                                                              │
│ ANNOTATION ACTIONS                                          │
│   Ctrl+Z ................ Undo                              │
│   Ctrl+Shift+Z .......... Redo                              │
│   Delete / Backspace .... Delete Selected                  │
│   Ctrl+S ................ Save and Close                    │
│   Escape ................ Cancel and Close                  │
└─────────────────────────────────────────────────────────────┘
```

---

### Troubleshooting Hotkeys

**Problem: Hotkeys aren't working**

- **Check 1:** Is the application running? Look for the Unbroken QA Capture icon in your system tray (notification area). If it's not there, launch the application from the Start menu.

- **Check 2:** Is another application using the same hotkey? Right-click the tray icon → Settings → Hotkeys. If you see a conflict warning, choose a different hotkey or close the conflicting application.

- **Check 3:** Are you in the correct state? Some hotkeys only work in specific states. For example, `F4` only works during bug capture mode, and `Print Screen` only works during an active session.

- **Check 4:** Windows hotkey registration limits. Windows has a limit on the number of global hotkeys that can be registered system-wide. If you're running many applications with global hotkeys, some may fail to register. Try closing other applications or restarting Windows.

**Problem: Print Screen opens Windows Snipping Tool but doesn't start a bug capture**

- This means the global hotkey registration failed. Check the application logs (Settings → Advanced → View Logs) for errors like "Failed to register hotkey: Print Screen already in use".

- Try customizing the bug capture hotkey to a different combination (Settings → Hotkeys → Bug Capture Trigger).

**Problem: Annotation window hotkeys don't work**

- Make sure the annotation window has focus (click on it if another window is on top).

- If a specific hotkey doesn't work (e.g., `Ctrl+Z`), check if your keyboard layout or language settings are interfering. Try switching to English (US) keyboard layout.

**Problem: I accidentally pressed a hotkey and started/ended something I didn't mean to**

- **Session accidentally started:** Press `Ctrl+Shift+Q` again immediately to end the session. You'll enter Review Mode with an empty session (no bugs). Just close the review window — no files are created for empty sessions.

- **Bug capture accidentally started:** Press `F4` to end the bug capture. An empty bug folder will be created but won't appear in Review Mode (only bugs with at least one screenshot are shown).

- **Notepad accidentally opened:** Press the same hotkey again (`Ctrl+Shift+N` or `Ctrl+Shift+M`) to close the notepad. Any text you typed is auto-saved.

---

### Accessibility Notes

**For users who cannot use standard keyboard shortcuts:**

- All global hotkeys can be triggered via the system tray menu (right-click the Unbroken icon)
- The main application window provides buttons for all actions (no hotkeys required)
- Mouse-only operation is fully supported — hotkeys are a convenience, not a requirement

**For users with motor impairments:**

- Consider changing hotkeys to combinations that are easier for you to press (e.g., single `F`-keys instead of multi-key combinations)
- The application supports Windows accessibility features (Sticky Keys, Filter Keys, etc.)
- Annotation tools can be selected by clicking the toolbar — hotkeys are optional

**For users with visual impairments:**

- The application works with Windows screen readers (Narrator, JAWS, NVDA)
- All hotkey actions provide audio feedback (configurable in Settings → Accessibility)
- High-contrast mode is supported (Settings → Appearance → High Contrast)

---

**Related Sections:**
- [Getting Started → First-Run Setup Wizard → Step 4: Hotkeys](#step-4-hotkeys) — Initial hotkey configuration
- [Settings Reference → Hotkeys](#) — Detailed hotkey customization options (see Settings Reference section)
- [Troubleshooting](#) — More solutions for common hotkey issues (see Troubleshooting section)
