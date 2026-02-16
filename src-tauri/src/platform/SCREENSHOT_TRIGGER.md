# Windows Screenshot Trigger Implementation

This document describes the implementation of the Windows Snipping Tool trigger functionality for Print Screen hotkey integration.

## Overview

When the user presses the Print Screen hotkey, the application needs to launch the Windows Snipping Tool without stealing focus from the application under test. The implementation uses multiple fallback methods to ensure reliability across different Windows 11 configurations.

## Implementation Details

### Trigger Methods (in fallback order)

The `trigger_screenshot()` method attempts three different approaches in sequence:

#### 1. URI Scheme (`ms-screenclip:`)

**Method:** Launch the `ms-screenclip:` URI using Windows shell command

**Availability:** Windows 10 1809+ and Windows 11

**Advantages:**
- Recommended Microsoft method for triggering Snip & Sketch
- Launches directly into screenshot mode
- No focus stealing - overlay appears immediately

**Implementation:**
```rust
Command::new("cmd")
    .args(["/C", "start", "ms-screenclip:"])
    .creation_flags(0x08000000) // CREATE_NO_WINDOW
    .spawn()
```

**Potential Failures:**
- URI scheme not registered (rare on Windows 11)
- Snipping Tool disabled by group policy

#### 2. Process Launch (`SnippingTool.exe`)

**Method:** Spawn `SnippingTool.exe` process

**Availability:** Windows 10 and Windows 11

**Advantages:**
- Direct process launch
- Works even if URI scheme is broken

**Implementation:**
```rust
Command::new("SnippingTool.exe")
    .creation_flags(0x08000000) // CREATE_NO_WINDOW
    .spawn()
```

**Potential Failures:**
- Snipping Tool not in PATH
- Executable renamed or moved
- Launches main window instead of direct capture mode

#### 3. Key Simulation (`Win+Shift+S`)

**Method:** Simulate keyboard input using Windows SendInput API

**Availability:** All Windows 10/11 versions with Snip & Sketch

**Advantages:**
- Guaranteed to work if the keyboard shortcut is enabled
- System-level API, very reliable

**Implementation:**
```rust
unsafe {
    // Press Win, Shift, S (in order)
    // Release S, Shift, Win (in reverse order)
    SendInput(&inputs, std::mem::size_of::<INPUT>() as i32)
}
```

**Potential Failures:**
- User has disabled Win+Shift+S in settings
- Another application has intercepted the hotkey

**Dependencies:**
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.58", features = ["Win32_UI_Input_KeyboardAndMouse"] }
```

### Focus Management

All three methods are designed to avoid focus stealing:

1. **URI/Process methods** use `CREATE_NO_WINDOW` flag to prevent console window flash
2. **Key simulation** triggers the system overlay, which appears on top without changing foreground window
3. No explicit window activation - the screenshot tool manages its own overlay

## Testing

### Unit Tests

Location: `src-tauri/src/platform/windows.rs` (module `tests`)

**Tests:**
- `test_trigger_screenshot_attempts_multiple_methods` - Verifies fallback chain
- `test_trigger_via_uri_does_not_panic` - URI method doesn't crash
- `test_trigger_via_process_does_not_panic` - Process method doesn't crash
- `test_trigger_via_keysim_does_not_panic` - Key simulation doesn't crash
- `test_trigger_screenshot_not_implemented_on_non_windows` - Platform checks

### Integration Tests

Location: `src-tauri/tests/screenshot_trigger_integration.rs`

**Tests:**
- `test_trigger_screenshot_integration` - End-to-end trigger test
- `test_file_watcher_detects_triggered_screenshots` - Verify file watcher integration (manual)
- `test_multiple_screenshot_triggers` - Rapid successive triggers
- `test_trigger_fallback_chain` - All methods return valid results

**Manual Tests:**
Some integration tests require manual verification:
```bash
cargo test --test screenshot_trigger_integration -- --ignored --nocapture
```

### Manual Verification on Windows 11

1. Build and run the application
2. Press the Print Screen hotkey
3. Verify:
   - Snipping Tool overlay appears immediately
   - Focus remains on the tested application
   - User can select screenshot region
   - Screenshot saves to configured folder
   - File watcher detects the new file

## Error Handling

All trigger methods return well-formed `Result` types:

- **Success:** `Ok(())` - Screenshot tool triggered
- **Failure:** `Err(PlatformError::ScreenshotTriggerError { method, message })`
  - `method`: Which method failed ("uri", "process", "keysim", "all")
  - `message`: Descriptive error message

If all three methods fail, the error indicates "all methods failed".

## Architecture Integration

### Flow

1. User presses Print Screen
2. `HotkeyManager` emits `hotkey-start-bug-capture` event
3. Frontend calls `start_bug_capture()` Tauri command
4. `SessionManager` calls `trigger_screenshot()` on `CaptureBridge`
5. `WindowsCaptureBridge::trigger_screenshot()` attempts methods in fallback order
6. Snipping Tool launches (overlay mode)
7. User completes screenshot
8. File watcher detects saved screenshot
9. Screenshot associated with active bug capture

### No Focus Stealing

The implementation ensures the application under test maintains focus:

- All methods launch the screenshot tool in overlay mode
- No explicit window activation API calls
- `CREATE_NO_WINDOW` prevents console flash
- System overlay (Win+Shift+S style) appears on top of all windows without changing Z-order

## Platform Support

- **Windows 11:** ✅ Fully supported (all three methods)
- **Windows 10 1809+:** ✅ Fully supported (all three methods)
- **Windows 10 (older):** ⚠️ Partial support (URI may not work, fallback to process/keysim)
- **macOS/Linux:** ❌ Not implemented (returns `NotImplemented` error)

## Known Limitations

1. **Group Policy Restrictions:** If Snipping Tool is disabled by corporate policy, all methods will fail
2. **Custom Hotkey Rebinding:** If user has changed Win+Shift+S to a different app, keysim method will fail
3. **UI Automation Tools:** Some screen readers or automation tools may interfere with SendInput
4. **Containerized Environments:** CI environments cannot test actual screenshot capture (manual verification required)

## Future Enhancements

- Registry redirect implementation (currently returns `NotImplemented`)
- Screenshot output folder configuration
- macOS support using `screencapture -i` CLI
- Retry logic with exponential backoff for transient failures
