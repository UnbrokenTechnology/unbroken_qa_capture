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
