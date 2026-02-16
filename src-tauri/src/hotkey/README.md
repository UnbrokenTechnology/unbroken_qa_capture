# Global Hotkey Module

This module provides global hotkey registration and management for the Unbroken QA Capture application using Tauri's `global-shortcut` plugin.

## Overview

The hotkey module allows users to trigger key application functions from anywhere in the system without needing to focus the application window. This is essential for QA workflows where the user is testing another application and needs to quickly capture bugs or start/end sessions.

## Features

- **Configurable hotkeys**: All shortcuts can be customized by the user
- **Default shortcuts**: Sensible defaults for common QA workflows
- **Graceful error handling**: Registration failures are logged and don't crash the app
- **No focus stealing**: Hotkeys trigger events but don't steal focus from the app under test
- **Session integration**: Direct integration with the Session Manager

## Default Hotkeys

| Action | Default Shortcut | Event Emitted |
|--------|------------------|---------------|
| Toggle Session | `Ctrl+Shift+Q` | `hotkey-toggle-session` |
| Start Bug Capture | `PrintScreen` | `hotkey-start-bug-capture` |
| End Bug Capture | `F4` | `hotkey-end-bug-capture` |
| Open Quick Notepad | `Ctrl+Shift+N` | `hotkey-open-quick-notepad` |
| Open Session Notepad | `Ctrl+Shift+M` | `hotkey-open-session-notepad` |

## Architecture

### HotkeyAction Enum

Defines all available hotkey actions. Each action has:
- **Event name**: The Tauri event to emit when triggered
- **Description**: Human-readable description for UI display

### HotkeyConfig Struct

Stores the mapping between actions and their keyboard shortcuts. Serializable for storage in settings.

### HotkeyManager

Manages the lifecycle of hotkey registration:
- **Registration**: Registers all configured shortcuts with the OS
- **Unregistration**: Cleanly removes shortcuts when updating config
- **Error handling**: Collects and reports registration failures
- **State tracking**: Tracks which shortcuts are currently registered

## Integration with Application

The hotkey manager is initialized in `lib.rs` during app setup:

1. Creates a `HotkeyManager` with default config
2. Registers all hotkeys with the Tauri global-shortcut plugin
3. Logs any registration failures to stderr
4. Stores the manager in a global static for access from Tauri commands

Registration failures are non-fatal - the app will continue to work with whatever hotkeys successfully registered.

## Tauri Commands

The following commands are exposed to the frontend:

### `get_hotkey_config() -> Result<HotkeyConfig, String>`

Returns the current hotkey configuration.

### `update_hotkey_config(config: HotkeyConfig) -> Result<Vec<String>, String>`

Updates the hotkey configuration and re-registers all shortcuts. Returns a list of error messages for any shortcuts that failed to register.

### `is_hotkey_registered(shortcut: String) -> Result<bool, String>`

Checks if a specific shortcut string is currently registered.

## Event Flow

1. User presses registered hotkey (e.g., `PrintScreen`)
2. OS notifies Tauri global-shortcut plugin
3. Plugin invokes registered callback
4. Callback emits Tauri event (e.g., `hotkey-start-bug-capture`)
5. Frontend Vue components listen for these events
6. Frontend invokes appropriate Session Manager commands

This design ensures no focus stealing - events are emitted but the UI doesn't force itself to the foreground.

## Error Handling

Registration can fail for several reasons:
- **Invalid shortcut format**: The shortcut string can't be parsed
- **Already registered**: Another application has claimed the shortcut
- **OS restrictions**: Some shortcuts are reserved by the OS

When registration fails:
1. Error is logged to stderr with details
2. Manager continues registering other shortcuts
3. Failed shortcuts are excluded from the "registered" list
4. Error messages are collected and can be shown to the user

The application remains functional with partial hotkey registration.

## Testing

The module includes comprehensive unit tests covering:
- Action enum serialization and event name mapping
- Config creation, serialization, and cloning
- Manager initialization with default and custom configs
- Registration state tracking
- Edge cases (empty config, duplicate shortcuts, etc.)

Integration tests verify:
- Manager initialization flows
- Config update workflows
- Event name and description mappings
- Serialization roundtrips

Tests are in `hotkey.rs` (unit tests) and `hotkey_tests.rs` (integration tests).

## Future Enhancements

Potential improvements:
- **Conflict detection**: Warn users if their custom shortcut is already registered by another app
- **Shortcut recorder**: UI widget to record keyboard input for custom shortcuts
- **Platform-specific defaults**: Different defaults for Windows/Mac/Linux
- **Shortcut profiles**: Save and load different sets of shortcuts for different workflows
