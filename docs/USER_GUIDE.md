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
