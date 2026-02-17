mod template;
pub mod database;
pub mod platform;
pub mod session_manager;
mod session_summary;
mod session_json;
mod hotkey;
mod claude_cli;
mod ticketing;

#[cfg(test)]
mod hotkey_tests;

use std::sync::{Arc, Mutex};
use template::TemplateManager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItemBuilder};
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, Emitter, AppHandle};
use session_manager::{SessionManager, EventEmitter, RealFileSystem};
use hotkey::{HotkeyManager, HotkeyConfig};
use ticketing::{LinearIntegration, TicketingIntegration};

// Global template manager
static TEMPLATE_MANAGER: Mutex<Option<TemplateManager>> = Mutex::new(None);

// Global session manager
static SESSION_MANAGER: Mutex<Option<Arc<SessionManager>>> = Mutex::new(None);

// Global hotkey manager
static HOTKEY_MANAGER: Mutex<Option<Arc<HotkeyManager>>> = Mutex::new(None);

// Global tray icon (must persist for app lifetime or it gets dropped/destroyed)
static TRAY_ICON: Mutex<Option<TrayIcon>> = Mutex::new(None);

// Global ticketing integration
static TICKETING_INTEGRATION: Mutex<Option<Arc<dyn TicketingIntegration>>> = Mutex::new(None);

// Global capture bridge (platform-specific screenshot/file-watcher implementation)
static CAPTURE_BRIDGE: Mutex<Option<Box<dyn platform::CaptureBridge>>> = Mutex::new(None);

// Active file watcher handle (None when no session is active)
static ACTIVE_WATCHER: Mutex<Option<platform::WatcherHandle>> = Mutex::new(None);

// Original Snipping Tool screenshot folder before redirect (for restoration on session end)
static ORIGINAL_SCREENSHOT_FOLDER: Mutex<Option<std::path::PathBuf>> = Mutex::new(None);

// Tauri event emitter implementation
struct TauriEventEmitter {
    app_handle: Arc<Mutex<Option<AppHandle>>>,
}

impl TauriEventEmitter {
    fn new() -> Self {
        TauriEventEmitter {
            app_handle: Arc::new(Mutex::new(None)),
        }
    }

    fn set_app_handle(&self, handle: AppHandle) {
        *self.app_handle.lock().unwrap() = Some(handle);
    }
}

impl EventEmitter for TauriEventEmitter {
    fn emit(&self, event: &str, payload: serde_json::Value) -> Result<(), String> {
        if let Some(handle) = self.app_handle.lock().unwrap().as_ref() {
            handle
                .emit(event, payload)
                .map_err(|e| format!("Failed to emit event: {}", e))
        } else {
            Err("App handle not initialized".to_string())
        }
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn set_custom_template_path(path: Option<String>) -> Result<(), String> {
    let mut manager_guard = TEMPLATE_MANAGER.lock().unwrap();

    if manager_guard.is_none() {
        *manager_guard = Some(TemplateManager::new());
    }

    let manager = manager_guard.as_mut().unwrap();
    let path_buf = path.map(std::path::PathBuf::from);
    manager.set_custom_template_path(path_buf)
}

#[tauri::command]
fn render_bug_template(bug_data: serde_json::Value) -> Result<String, String> {
    let mut manager_guard = TEMPLATE_MANAGER.lock().unwrap();

    if manager_guard.is_none() {
        *manager_guard = Some(TemplateManager::new());
    }

    let manager = manager_guard.as_ref().unwrap();

    // Convert JSON to BugData
    let bug: template::BugData = serde_json::from_value(bug_data)
        .map_err(|e| format!("Failed to parse bug data: {}", e))?;

    manager.render(&bug)
}

#[tauri::command]
fn reload_template() -> Result<(), String> {
    let manager_guard = TEMPLATE_MANAGER.lock().unwrap();

    if let Some(manager) = manager_guard.as_ref() {
        manager.reload_template()
    } else {
        Err("Template manager not initialized".to_string())
    }
}

#[tauri::command]
fn get_template_source() -> Result<String, String> {
    // Check if there's a custom template path set
    let manager_guard = TEMPLATE_MANAGER.lock().unwrap();

    if manager_guard.is_none() {
        // Return default template
        return Ok(template::DEFAULT_TEMPLATE.to_string());
    }

    let manager = manager_guard.as_ref().unwrap();

    // If custom template exists, read it; otherwise return default
    if let Some(custom_path) = &manager.custom_template_path {
        std::fs::read_to_string(custom_path)
            .or_else(|_| Ok(template::DEFAULT_TEMPLATE.to_string()))
    } else {
        Ok(template::DEFAULT_TEMPLATE.to_string())
    }
}

#[tauri::command]
fn save_custom_template(content: String, app: tauri::AppHandle) -> Result<String, String> {
    // Get app data directory
    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });

    // Create templates directory if it doesn't exist
    let templates_dir = data_dir.join("templates");
    std::fs::create_dir_all(&templates_dir)
        .map_err(|e| format!("Failed to create templates directory: {}", e))?;

    // Save custom template
    let custom_template_path = templates_dir.join("custom_template.md");
    std::fs::write(&custom_template_path, &content)
        .map_err(|e| format!("Failed to save custom template: {}", e))?;

    // Update template manager to use custom template
    let mut manager_guard = TEMPLATE_MANAGER.lock().unwrap();

    if manager_guard.is_none() {
        *manager_guard = Some(TemplateManager::new());
    }

    let manager = manager_guard.as_mut().unwrap();
    manager.set_custom_template_path(Some(custom_template_path.clone()))?;

    Ok(custom_template_path.to_string_lossy().to_string())
}

#[tauri::command]
fn reset_template_to_default() -> Result<(), String> {
    let mut manager_guard = TEMPLATE_MANAGER.lock().unwrap();

    if manager_guard.is_none() {
        *manager_guard = Some(TemplateManager::new());
    }

    let manager = manager_guard.as_mut().unwrap();
    manager.set_custom_template_path(None)
}

#[tauri::command]
fn get_template_path(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let manager_guard = TEMPLATE_MANAGER.lock().unwrap();

    if let Some(manager) = manager_guard.as_ref() {
        if let Some(custom_path) = &manager.custom_template_path {
            return Ok(Some(custom_path.to_string_lossy().to_string()));
        }
    }

    // Return path to default template (embedded, but we'll create a temp copy for editing)
    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let templates_dir = data_dir.join("templates");
    std::fs::create_dir_all(&templates_dir).ok();

    let default_template_path = templates_dir.join("default_template.md");

    // Write default template to file if it doesn't exist
    if !default_template_path.exists() {
        std::fs::write(&default_template_path, template::DEFAULT_TEMPLATE)
            .map_err(|e| format!("Failed to write default template: {}", e))?;
    }

    Ok(Some(default_template_path.to_string_lossy().to_string()))
}

#[tauri::command]
async fn open_template_in_editor(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_opener::OpenerExt;

    // Get the template path
    let template_path = get_template_path(app.clone())?
        .ok_or("No template path available")?;

    // Open in system default editor
    app.opener()
        .open_path(&template_path, None::<&str>)
        .map_err(|e| format!("Failed to open template in editor: {}", e))?;

    Ok(())
}

/// Helper function to read bug data from a folder and render it using the template
fn read_and_render_bug(folder_path: &str) -> Result<String, String> {
    use std::path::Path;

    // Read bug data from the folder
    let folder = Path::new(folder_path);
    if !folder.exists() || !folder.is_dir() {
        return Err(format!("Bug folder does not exist: {}", folder_path));
    }

    // Read metadata.json
    let metadata_path = folder.join("metadata.json");
    if !metadata_path.exists() {
        return Err(format!(
            "metadata.json not found in folder: {}",
            folder_path
        ));
    }

    let metadata_content = std::fs::read_to_string(&metadata_path)
        .map_err(|e| format!("Failed to read metadata.json: {}", e))?;

    let bug_data: serde_json::Value = serde_json::from_str(&metadata_content)
        .map_err(|e| format!("Failed to parse metadata.json: {}", e))?;

    // Get TemplateManager and render the bug
    let mut manager_guard = TEMPLATE_MANAGER.lock().unwrap();

    if manager_guard.is_none() {
        *manager_guard = Some(TemplateManager::new());
    }

    let manager = manager_guard.as_ref().unwrap();

    // Convert JSON to BugData
    let bug: template::BugData = serde_json::from_value(bug_data)
        .map_err(|e| format!("Failed to parse bug data: {}", e))?;

    // Render the template
    manager.render(&bug)
}

#[tauri::command]
async fn copy_bug_to_clipboard(
    folder_path: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    // Read and render the bug data
    let rendered_markdown = read_and_render_bug(&folder_path)?;

    // Copy to clipboard using Tauri clipboard plugin
    app_handle
        .clipboard()
        .write_text(rendered_markdown)
        .map_err(|e| format!("Failed to copy to clipboard: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn open_bug_folder(folder_path: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    use std::path::Path;
    use tauri_plugin_opener::OpenerExt;

    // Validate that the path exists and is a directory
    let path = Path::new(&folder_path);
    if !path.exists() {
        return Err(format!("Bug folder does not exist: {}", folder_path));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path));
    }

    // Open the folder in the system file manager
    app_handle
        .opener()
        .open_path(&folder_path, None::<&str>)
        .map_err(|e| format!("Failed to open bug folder: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn open_session_folder(
    folder_path: String,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    use std::path::Path;
    use tauri_plugin_opener::OpenerExt;

    // Validate that the path exists and is a directory
    let path = Path::new(&folder_path);
    if !path.exists() {
        return Err(format!("Session folder does not exist: {}", folder_path));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", folder_path));
    }

    // Open the folder in the system file manager
    app_handle
        .opener()
        .open_path(&folder_path, None::<&str>)
        .map_err(|e| format!("Failed to open session folder: {}", e))?;

    Ok(())
}

/// Load the embedded tray icon PNG for the given state.
///
/// PRD Section 14 (Iconography) specifies:
/// - idle:   gray/neutral circle
/// - active: green indicator
/// - bug:    red indicator
/// - review: blue indicator
///
/// Icons are 32x32 8-bit RGBA PNGs embedded at compile time.
/// Decode a PNG byte slice into raw RGBA pixels.
fn decode_png_rgba(png_bytes: &[u8]) -> Result<(Vec<u8>, u32, u32), String> {
    use std::io::Cursor;
    let decoder = png::Decoder::new(Cursor::new(png_bytes));
    let mut reader = decoder.read_info().map_err(|e| format!("PNG decode error: {}", e))?;
    let mut buf = vec![0u8; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).map_err(|e| format!("PNG frame error: {}", e))?;
    // Ensure RGBA output
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf[..info.buffer_size()].to_vec(),
        png::ColorType::Rgb => {
            // Expand RGB to RGBA by adding full-opacity alpha channel
            let rgb = &buf[..info.buffer_size()];
            let mut rgba = Vec::with_capacity(rgb.len() / 3 * 4);
            for chunk in rgb.chunks(3) {
                rgba.extend_from_slice(chunk);
                rgba.push(255);
            }
            rgba
        }
        ct => return Err(format!("Unsupported PNG color type: {:?}", ct)),
    };
    Ok((rgba, info.width, info.height))
}

/// Load the embedded tray icon PNG for the given state.
///
/// PRD Section 14 (Iconography) specifies:
/// - idle:   gray/neutral circle
/// - active: green indicator
/// - bug:    red indicator
/// - review: blue indicator
///
/// Icons are 32x32 8-bit RGBA PNGs embedded at compile time.
fn tray_icon_for_state(state: &str) -> Result<Image<'static>, String> {
    let png_bytes: &[u8] = match state {
        "active" => include_bytes!("../icons/tray/tray-active-32.png"),
        "bug"    => include_bytes!("../icons/tray/tray-bug-32.png"),
        "review" => include_bytes!("../icons/tray/tray-review-32.png"),
        _        => include_bytes!("../icons/tray/tray-idle-32.png"),  // idle + unknown
    };
    let (rgba, width, height) = decode_png_rgba(png_bytes)?;
    Ok(Image::new_owned(rgba, width, height))
}

#[tauri::command]
async fn update_tray_icon(state: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    update_tray_menu(state, None, app_handle).await
}

/// Rebuild the tray context menu to reflect the current app state.
///
/// PRD Section 13 specifies different menus per state:
/// - Idle: 'Start Session', 'Open App', 'Settings', 'Quit'
/// - Active Session: 'End Session', 'Open App', 'Quit'
/// - Bug Capture: 'End Bug Capture (F4)', 'End Session', 'Open App'
/// - Review: 'Open Review', 'Quit'
#[tauri::command]
async fn update_tray_menu(state: String, bug_id: Option<String>, app_handle: tauri::AppHandle) -> Result<(), String> {
    let Some(tray) = app_handle.tray_by_id("main-tray") else {
        return Ok(());
    };

    let menu = Menu::new(&app_handle)
        .map_err(|e| format!("Failed to create menu: {}", e))?;

    match state.as_str() {
        "idle" => {
            let start = MenuItemBuilder::new("Start Session")
                .id("start-session").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let open = MenuItemBuilder::new("Open App")
                .id("open-main-window").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let settings = MenuItemBuilder::new("Settings")
                .id("settings").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let help = MenuItemBuilder::new("Help / User Guide")
                .id("help").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let quit = MenuItemBuilder::new("Quit")
                .id("quit").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            menu.append_items(&[&start, &open, &settings, &help, &quit])
                .map_err(|e| format!("Failed to append menu items: {}", e))?;
        }
        "active" => {
            let end = MenuItemBuilder::new("End Session")
                .id("end-session").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let open = MenuItemBuilder::new("Open App")
                .id("open-main-window").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let help = MenuItemBuilder::new("Help / User Guide")
                .id("help").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let quit = MenuItemBuilder::new("Quit")
                .id("quit").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            menu.append_items(&[&end, &open, &help, &quit])
                .map_err(|e| format!("Failed to append menu items: {}", e))?;
        }
        "bug" => {
            let label = if let Some(id) = &bug_id {
                format!("End Bug Capture {} (F4)", id)
            } else {
                "End Bug Capture (F4)".to_string()
            };
            let end_bug = MenuItemBuilder::new(&label)
                .id("end-bug-capture").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let end_session = MenuItemBuilder::new("End Session")
                .id("end-session").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let open = MenuItemBuilder::new("Open App")
                .id("open-main-window").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let help = MenuItemBuilder::new("Help / User Guide")
                .id("help").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            menu.append_items(&[&end_bug, &end_session, &open, &help])
                .map_err(|e| format!("Failed to append menu items: {}", e))?;
        }
        "review" => {
            let open_review = MenuItemBuilder::new("Open Review")
                .id("open-review").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let help = MenuItemBuilder::new("Help / User Guide")
                .id("help").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let quit = MenuItemBuilder::new("Quit")
                .id("quit").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            menu.append_items(&[&open_review, &help, &quit])
                .map_err(|e| format!("Failed to append menu items: {}", e))?;
        }
        _ => {
            // Unknown state — fall back to idle menu
            let open = MenuItemBuilder::new("Open App")
                .id("open-main-window").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let help = MenuItemBuilder::new("Help / User Guide")
                .id("help").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            let quit = MenuItemBuilder::new("Quit")
                .id("quit").enabled(true).build(&app_handle)
                .map_err(|e| format!("Menu item error: {}", e))?;
            menu.append_items(&[&open, &help, &quit])
                .map_err(|e| format!("Failed to append menu items: {}", e))?;
        }
    }

    tray.set_menu(Some(menu))
        .map_err(|e| format!("Failed to set tray menu: {}", e))?;

    // Update the tray icon image to reflect the new state (PRD Section 14)
    let icon = tray_icon_for_state(state.as_str())?;
    tray.set_icon(Some(icon))
        .map_err(|e| format!("Failed to set tray icon: {}", e))?;

    // Also emit event so frontend can react if needed
    app_handle
        .emit("tray-state-changed", &state)
        .map_err(|e| format!("Failed to emit tray state event: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn update_tray_tooltip(tooltip: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    if let Some(tray) = app_handle.tray_by_id("main-tray") {
        tray.set_tooltip(Some(tooltip))
            .map_err(|e| format!("Failed to set tray tooltip: {}", e))?;
    }
    Ok(())
}

#[tauri::command]
async fn get_bug_notes(_bug_id: String, folder_path: String) -> Result<String, String> {
    use std::path::Path;

    // First try to read from notes.md file
    let notes_file = Path::new(&folder_path).join("notes.md");
    if notes_file.exists() {
        std::fs::read_to_string(&notes_file)
            .map_err(|e| format!("Failed to read notes.md: {}", e))
    } else {
        // Return empty string if file doesn't exist yet
        Ok(String::new())
    }
}

#[tauri::command]
async fn update_bug_notes(
    _bug_id: String,
    folder_path: String,
    notes: String,
) -> Result<(), String> {
    use std::path::Path;

    // Ensure the folder exists
    let bug_folder = Path::new(&folder_path);
    if !bug_folder.exists() {
        std::fs::create_dir_all(bug_folder)
            .map_err(|e| format!("Failed to create bug folder: {}", e))?;
    }

    // Write notes to notes.md file
    let notes_file = bug_folder.join("notes.md");
    std::fs::write(&notes_file, notes)
        .map_err(|e| format!("Failed to write notes.md: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn get_session_notes(_session_id: String, folder_path: String) -> Result<String, String> {
    use std::path::Path;

    // Try to read from session-notes.md file
    let notes_file = Path::new(&folder_path).join("session-notes.md");
    if notes_file.exists() {
        std::fs::read_to_string(&notes_file)
            .map_err(|e| format!("Failed to read session-notes.md: {}", e))
    } else {
        // Return empty string if file doesn't exist yet
        Ok(String::new())
    }
}

#[tauri::command]
async fn update_session_notes(
    _session_id: String,
    folder_path: String,
    notes: String,
) -> Result<(), String> {
    use std::path::Path;

    // Ensure the folder exists
    let session_folder = Path::new(&folder_path);
    if !session_folder.exists() {
        std::fs::create_dir_all(session_folder)
            .map_err(|e| format!("Failed to create session folder: {}", e))?;
    }

    // Write notes to session-notes.md file
    let notes_file = session_folder.join("session-notes.md");
    std::fs::write(&notes_file, notes)
        .map_err(|e| format!("Failed to write session-notes.md: {}", e))?;

    Ok(())
}

#[tauri::command]
async fn open_session_notes_window(app: tauri::AppHandle) -> Result<(), String> {
    let window_label = "session-notes";

    // If already open, focus it instead of creating a new one
    if let Some(existing) = app.get_webview_window(window_label) {
        existing.show().map_err(|e| format!("Failed to show session notes window: {}", e))?;
        existing.set_focus().map_err(|e| format!("Failed to focus session notes window: {}", e))?;
        return Ok(());
    }

    tauri::WebviewWindowBuilder::new(
        &app,
        window_label,
        tauri::WebviewUrl::App("/session-notes".into()),
    )
    .title("Session Notes")
    .inner_size(450.0, 380.0)
    .min_inner_size(300.0, 250.0)
    .resizable(true)
    .decorations(true)
    .always_on_top(true)
    .focused(true)
    .build()
    .map_err(|e| format!("Failed to create session notes window: {}", e))?;

    Ok(())
}

// ─── Session Manager Commands ────────────────────────────────────────────

/// Determine capture type and generate PRD-compliant file name.
/// Screenshots: capture-{NNN}.png, Videos: recording-{NNN}.mp4 (or .webm/.mkv).
fn make_capture_filename(source_path: &std::path::Path, capture_number: u32) -> (String, database::CaptureType) {
    use database::CaptureType;
    let extension = source_path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("png")
        .to_lowercase();
    match extension.as_str() {
        "mp4" | "webm" | "mkv" => (
            format!("recording-{:03}.{}", capture_number, extension),
            CaptureType::Video,
        ),
        ext => (
            format!("capture-{:03}.{}", capture_number, ext),
            CaptureType::Screenshot,
        ),
    }
}

/// Count existing captures in a directory to determine the next sequential number.
fn next_capture_number(dir: &std::path::Path) -> u32 {
    let count = std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .filter_map(|e| e.ok())
                .filter(|e| {
                    let name = e.file_name();
                    let s = name.to_string_lossy();
                    s.starts_with("capture-") || s.starts_with("recording-")
                })
                .count()
        })
        .unwrap_or(0);
    (count as u32) + 1
}

/// Copy a file detected with no active bug into the session's _unsorted/ directory,
/// rename it per PRD conventions (capture-NNN.png / recording-NNN.mp4),
/// create a Capture record with bug_id=None, and emit a capture:file-detected event.
fn route_to_unsorted(
    app_handle: &AppHandle,
    file_path: &std::path::Path,
    session_folder: &str,
    session_id: &str,
    file_size: u64,
    _is_video: bool,
) {
    use database::{Database, CaptureOps, CaptureRepository, Capture};
    use uuid::Uuid;
    use chrono::Utc;

    let unsorted_dir = std::path::Path::new(session_folder).join("_unsorted");

    // Ensure _unsorted directory exists (may have been deleted by the OS or tests)
    if let Err(e) = std::fs::create_dir_all(&unsorted_dir) {
        eprintln!("Warning: Failed to create _unsorted directory: {}", e);
        return;
    }

    // Generate PRD-compliant file name
    let capture_num = next_capture_number(&unsorted_dir);
    let (file_name, capture_type) = make_capture_filename(file_path, capture_num);

    // Destination path in _unsorted/ with new name
    let dest_path = unsorted_dir.join(&file_name);

    // Copy (not move) so the original location is preserved if needed
    if let Err(e) = std::fs::copy(file_path, &dest_path) {
        eprintln!("Warning: Failed to copy file to _unsorted/: {}", e);
        return;
    }

    // Get DB path
    let db_path = app_handle.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    }).join("qa_capture.db");

    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Warning: Failed to open database for unsorted capture: {}", e);
            return;
        }
    };

    let repo = CaptureRepository::new(db.connection());
    let capture_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let capture = Capture {
        id: capture_id.clone(),
        bug_id: None,
        session_id: session_id.to_string(),
        file_name: file_name.clone(),
        file_path: dest_path.to_string_lossy().to_string(),
        file_type: capture_type,
        annotated_path: None,
        file_size_bytes: Some(file_size as i64),
        is_console_capture: false,
        parsed_content: None,
        created_at: now,
    };

    if let Err(e) = repo.create(&capture) {
        eprintln!("Warning: Failed to create unsorted capture record: {}", e);
        return;
    }

    // Emit event so the frontend can update the Unsorted Captures section
    let _ = app_handle.emit("capture:file-detected", serde_json::json!({
        "filePath": dest_path.to_string_lossy().to_string(),
        "captureId": capture_id,
        "sessionId": session_id,
        "bugId": null,
        "type": capture.file_type.as_str(),
    }));
}

/// Copy a file detected while a bug is active into the bug's folder,
/// rename it per PRD conventions (capture-NNN.png / recording-NNN.mp4),
/// create a Capture record with bug_id set, and emit a capture:file-detected event.
fn route_to_bug(
    app_handle: &AppHandle,
    file_path: &std::path::Path,
    session_id: &str,
    bug_id: &str,
    bug_folder: &str,
    file_size: u64,
) {
    use database::{Database, CaptureOps, CaptureRepository, Capture};
    use uuid::Uuid;
    use chrono::Utc;

    let bug_dir = std::path::Path::new(bug_folder);

    // Ensure bug folder exists
    if let Err(e) = std::fs::create_dir_all(bug_dir) {
        eprintln!("Warning: Failed to create bug directory: {}", e);
        return;
    }

    // Generate PRD-compliant file name
    let capture_num = next_capture_number(bug_dir);
    let (file_name, capture_type) = make_capture_filename(file_path, capture_num);

    let dest_path = bug_dir.join(&file_name);

    // Copy file to bug folder with new name
    if let Err(e) = std::fs::copy(file_path, &dest_path) {
        eprintln!("Warning: Failed to copy file to bug folder: {}", e);
        return;
    }

    // Get DB path
    let db_path = app_handle.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    }).join("qa_capture.db");

    let db = match Database::open(&db_path) {
        Ok(db) => db,
        Err(e) => {
            eprintln!("Warning: Failed to open database for bug capture: {}", e);
            return;
        }
    };

    let repo = CaptureRepository::new(db.connection());
    let capture_id = Uuid::new_v4().to_string();
    let now = Utc::now().to_rfc3339();

    let capture = Capture {
        id: capture_id.clone(),
        bug_id: Some(bug_id.to_string()),
        session_id: session_id.to_string(),
        file_name: file_name.clone(),
        file_path: dest_path.to_string_lossy().to_string(),
        file_type: capture_type,
        annotated_path: None,
        file_size_bytes: Some(file_size as i64),
        is_console_capture: false,
        parsed_content: None,
        created_at: now,
    };

    if let Err(e) = repo.create(&capture) {
        eprintln!("Warning: Failed to create bug capture record: {}", e);
        return;
    }

    // Emit event so the frontend can update the Bug Captures section
    let _ = app_handle.emit("capture:file-detected", serde_json::json!({
        "filePath": dest_path.to_string_lossy().to_string(),
        "captureId": capture_id,
        "sessionId": session_id,
        "bugId": bug_id,
        "type": capture.file_type.as_str(),
    }));
}

#[tauri::command]
fn start_session(app: tauri::AppHandle) -> Result<database::Session, String> {
    use std::sync::mpsc;

    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Session manager not initialized")?;
    let session = manager.start_session()?;

    // Watch _captures/ subdirectory — the PRD-specified temporary landing zone
    // where Snipping Tool saves files before they are sorted into bug subfolders.
    let folder_path = session.folder_path.clone();
    let captures_watch_path = std::path::Path::new(&folder_path)
        .join("_captures")
        .to_string_lossy()
        .to_string();
    drop(manager_guard); // Release lock before acquiring CAPTURE_BRIDGE lock

    let (tx, rx) = mpsc::channel();
    {
        let bridge_guard = CAPTURE_BRIDGE.lock().unwrap();
        if let Some(bridge) = bridge_guard.as_ref() {
            // Stop any existing watcher first
            {
                let mut active = ACTIVE_WATCHER.lock().unwrap();
                if let Some(old_handle) = active.take() {
                    let _ = bridge.stop_file_watcher(old_handle);
                }
            }

            match bridge.start_file_watcher(std::path::Path::new(&captures_watch_path), tx) {
                Ok(handle) => {
                    *ACTIVE_WATCHER.lock().unwrap() = Some(handle);

                    // Redirect Snipping Tool output to _captures/ so screenshots land there automatically.
                    // Best-effort: log warnings on failure but don't abort session start.
                    match bridge.redirect_screenshot_output(std::path::Path::new(&captures_watch_path)) {
                        Ok(original) => {
                            *ORIGINAL_SCREENSHOT_FOLDER.lock().unwrap() = Some(original);
                        }
                        Err(e) => {
                            eprintln!("Warning: Could not redirect Snipping Tool output folder: {}", e);
                        }
                    }

                    // Spawn background thread to sort capture events from _captures/
                    // into bug subfolders or _unsorted/, with PRD-compliant naming.
                    let app_handle = app.clone();
                    let session_folder = folder_path.clone();
                    std::thread::spawn(move || {
                        while let Ok(event) = rx.recv() {
                            match event {
                                platform::CaptureEvent::ScreenshotDetected { file_path, detected_at: _, file_size } => {
                                    let (active_bug_id, active_session_id) = {
                                        let guard = SESSION_MANAGER.lock().ok();
                                        let bug_id = guard.as_ref().and_then(|m| m.as_ref().and_then(|sm| sm.get_active_bug_id()));
                                        let session_id = guard.as_ref().and_then(|m| m.as_ref().and_then(|sm| sm.get_active_session_id()));
                                        (bug_id, session_id)
                                    };

                                    if let Some(bug_id) = active_bug_id {
                                        if let Some(session_id) = active_session_id {
                                            // Look up bug folder from DB
                                            let db_path = app_handle.path().app_data_dir()
                                                .unwrap_or_else(|_| std::env::current_dir().unwrap().join("data"))
                                                .join("qa_capture.db");
                                            if let Ok(db) = database::Database::open(&db_path) {
                                                use database::{BugOps, BugRepository};
                                                let repo = BugRepository::new(db.connection());
                                                if let Ok(Some(bug)) = repo.get(&bug_id) {
                                                    route_to_bug(&app_handle, &file_path, &session_id, &bug_id, &bug.folder_path, file_size);
                                                }
                                            }
                                        }
                                    } else if let Some(session_id) = active_session_id {
                                        route_to_unsorted(&app_handle, &file_path, &session_folder, &session_id, file_size, false);
                                    }
                                }
                                platform::CaptureEvent::VideoDetected { file_path, detected_at: _, file_size } => {
                                    let (active_bug_id, active_session_id) = {
                                        let guard = SESSION_MANAGER.lock().ok();
                                        let bug_id = guard.as_ref().and_then(|m| m.as_ref().and_then(|sm| sm.get_active_bug_id()));
                                        let session_id = guard.as_ref().and_then(|m| m.as_ref().and_then(|sm| sm.get_active_session_id()));
                                        (bug_id, session_id)
                                    };

                                    if let Some(bug_id) = active_bug_id {
                                        if let Some(session_id) = active_session_id {
                                            let db_path = app_handle.path().app_data_dir()
                                                .unwrap_or_else(|_| std::env::current_dir().unwrap().join("data"))
                                                .join("qa_capture.db");
                                            if let Ok(db) = database::Database::open(&db_path) {
                                                use database::{BugOps, BugRepository};
                                                let repo = BugRepository::new(db.connection());
                                                if let Ok(Some(bug)) = repo.get(&bug_id) {
                                                    route_to_bug(&app_handle, &file_path, &session_id, &bug_id, &bug.folder_path, file_size);
                                                }
                                            }
                                        }
                                    } else if let Some(session_id) = active_session_id {
                                        route_to_unsorted(&app_handle, &file_path, &session_folder, &session_id, file_size, true);
                                    }
                                }
                                platform::CaptureEvent::WatcherError { message } => {
                                    eprintln!("File watcher error: {}", message);
                                }
                            }
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Warning: Failed to start file watcher for _captures/ folder '{}': {}", captures_watch_path, e);
                }
            }
        }
    }

    Ok(session)
}

#[tauri::command]
async fn end_session(session_id: String) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let manager_guard = SESSION_MANAGER.lock().unwrap();
        let manager = manager_guard
            .as_ref()
            .ok_or("Session manager not initialized")?;
        let result = manager.end_session(&session_id);
        drop(manager_guard); // Release lock before acquiring CAPTURE_BRIDGE lock

        // Stop the active file watcher when the session ends
        {
            let bridge_guard = CAPTURE_BRIDGE.lock().unwrap();
            if let Some(bridge) = bridge_guard.as_ref() {
                let mut active = ACTIVE_WATCHER.lock().unwrap();
                if let Some(handle) = active.take() {
                    if let Err(e) = bridge.stop_file_watcher(handle) {
                        eprintln!("Warning: Failed to stop file watcher: {}", e);
                    }
                }

                // Restore the original Snipping Tool screenshot folder
                let original = ORIGINAL_SCREENSHOT_FOLDER.lock().unwrap().take();
                if let Some(original_path) = original {
                    if let Err(e) = bridge.restore_screenshot_output(&original_path) {
                        eprintln!("Warning: Failed to restore Snipping Tool output folder: {}", e);
                    }
                }
            }
        }

        result
    })
    .await
    .map_err(|e| format!("Task join error: {}", e))?
}

#[tauri::command]
fn resume_session(session_id: String) -> Result<database::Session, String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Session manager not initialized")?;
    manager.resume_session(&session_id)
}

#[tauri::command]
fn start_bug_capture(session_id: String) -> Result<database::Bug, String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Session manager not initialized")?;
    manager.start_bug_capture(&session_id)
}

#[tauri::command]
fn end_bug_capture(bug_id: String) -> Result<(), String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Session manager not initialized")?;
    manager.end_bug_capture(&bug_id)
}

/// Resume capturing for an existing bug — sets its status back to 'capturing' and marks it as the active bug.
/// Used when the user wants to add more screenshots to a bug that was previously ended.
#[tauri::command]
fn resume_bug_capture(bug_id: String) -> Result<database::Bug, String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Session manager not initialized")?;
    manager.resume_bug_capture(&bug_id)
}

#[tauri::command]
fn get_active_session_id() -> Result<Option<String>, String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Session manager not initialized")?;
    Ok(manager.get_active_session_id())
}

#[tauri::command]
fn get_active_bug_id() -> Result<Option<String>, String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Session manager not initialized")?;
    Ok(manager.get_active_bug_id())
}

#[tauri::command]
fn get_session_summaries(app: tauri::AppHandle) -> Result<Vec<database::SessionSummary>, String> {
    use database::{Database, SessionRepository, SessionOps};

    let db_path = app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("qa_capture.db");

    let db = Database::new(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let repo = SessionRepository::new(db.connection());
    repo.get_summaries()
        .map_err(|e| format!("Failed to get session summaries: {}", e))
}

#[tauri::command]
fn get_active_session(app: tauri::AppHandle) -> Result<Option<database::Session>, String> {
    use database::{Database, SessionRepository, SessionOps};

    let db_path = app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("qa_capture.db");

    let db = Database::new(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let repo = SessionRepository::new(db.connection());
    repo.get_active_session()
        .map_err(|e| format!("Failed to get active session: {}", e))
}

#[tauri::command]
fn list_sessions(app: tauri::AppHandle) -> Result<Vec<database::Session>, String> {
    use database::{Database, SessionRepository, SessionOps};

    let db_path = app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("qa_capture.db");

    let db = Database::new(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let repo = SessionRepository::new(db.connection());
    repo.list()
        .map_err(|e| format!("Failed to list sessions: {}", e))
}

#[tauri::command]
fn update_session_status(session_id: String, status: String, app: tauri::AppHandle) -> Result<(), String> {
    use database::{Database, SessionRepository, SessionOps};

    let parsed_status = match status.as_str() {
        "active" => database::SessionStatus::Active,
        "ended" => database::SessionStatus::Ended,
        "reviewed" => database::SessionStatus::Reviewed,
        "synced" => database::SessionStatus::Synced,
        _ => return Err(format!("Invalid session status: {}", status)),
    };

    let db_path = app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("qa_capture.db");

    let db = Database::new(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let repo = SessionRepository::new(db.connection());
    repo.update_status(&session_id, parsed_status)
        .map_err(|e| format!("Failed to update session status: {}", e))
}

#[tauri::command]
fn get_bugs_by_session(session_id: String, app: tauri::AppHandle) -> Result<Vec<database::Bug>, String> {
    use database::{Database, BugRepository, BugOps};

    let db_path = app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?
        .join("qa_capture.db");

    let db = Database::new(&db_path)
        .map_err(|e| format!("Failed to open database: {}", e))?;

    let repo = BugRepository::new(db.connection());
    repo.list_by_session(&session_id)
        .map_err(|e| format!("Failed to get bugs for session: {}", e))
}

#[tauri::command]
fn generate_session_summary(session_id: String, include_ai_summary: bool) -> Result<String, String> {
    use session_summary::SessionSummaryGenerator;

    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Session manager not initialized")?;

    // Get db_path from the session manager's db_path field
    // We'll create a new generator with the same db_path
    let db_path = manager.db_path.clone();

    let generator = SessionSummaryGenerator::new(db_path);
    generator.generate_summary(&session_id, include_ai_summary)
}

// ─── Hotkey Manager Commands ─────────────────────────────────────────────

#[tauri::command]
fn get_hotkey_config() -> Result<HotkeyConfig, String> {
    let manager_guard = HOTKEY_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Hotkey manager not initialized")?;
    Ok(manager.get_config())
}

#[tauri::command]
fn update_hotkey_config(
    config: HotkeyConfig,
    app_handle: tauri::AppHandle,
) -> Result<Vec<String>, String> {
    use database::{Database, SettingsRepository, SettingsOps};

    let manager_guard = HOTKEY_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Hotkey manager not initialized")?;

    // Save config to database settings
    let data_dir = app_handle.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    manager.save_to_settings(&config, |key, value| {
        let db = Database::open(&db_path).map_err(|e| e.to_string())?;
        let repo = SettingsRepository::new(db.connection());
        repo.set(key, value).map_err(|e: rusqlite::Error| e.to_string())
    })?;

    // Update the runtime config and re-register hotkeys
    let results = manager.update_config(&app_handle, config);

    // Collect error messages
    let errors: Vec<String> = results
        .into_iter()
        .filter_map(|r| r.err())
        .collect();

    Ok(errors)
}

#[tauri::command]
fn is_hotkey_registered(shortcut: String) -> Result<bool, String> {
    let manager_guard = HOTKEY_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Hotkey manager not initialized")?;
    Ok(manager.is_registered(&shortcut))
}

// ─── Ticketing Integration Commands ──────────────────────────────────────

#[tauri::command]
fn ticketing_authenticate(credentials: ticketing::TicketingCredentials) -> Result<(), String> {
    let integration_guard = TICKETING_INTEGRATION.lock().unwrap();
    let integration = integration_guard
        .as_ref()
        .ok_or("Ticketing integration not initialized")?;

    integration
        .authenticate(&credentials)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ticketing_create_ticket(request: ticketing::CreateTicketRequest) -> Result<ticketing::CreateTicketResponse, String> {
    let integration_guard = TICKETING_INTEGRATION.lock().unwrap();
    let integration = integration_guard
        .as_ref()
        .ok_or("Ticketing integration not initialized")?;

    integration
        .create_ticket(&request)
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ticketing_check_connection() -> Result<ticketing::ConnectionStatus, String> {
    let integration_guard = TICKETING_INTEGRATION.lock().unwrap();
    let integration = integration_guard
        .as_ref()
        .ok_or("Ticketing integration not initialized")?;

    integration
        .check_connection()
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn ticketing_get_credentials(app: tauri::AppHandle) -> Result<Option<ticketing::TicketingCredentials>, String> {
    use database::{Database, SettingsRepository, SettingsOps};

    // Get database path
    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    // Open database
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = SettingsRepository::new(db.connection());

    // Get stored credentials
    let api_key = repo.get("ticketing.api_key").map_err(|e: rusqlite::Error| e.to_string())?;
    let team_id = repo.get("ticketing.team_id").map_err(|e: rusqlite::Error| e.to_string())?;
    let workspace_id = repo.get("ticketing.workspace_id").map_err(|e: rusqlite::Error| e.to_string())?;

    if let Some(key) = api_key {
        Ok(Some(ticketing::TicketingCredentials {
            api_key: key,
            team_id,
            workspace_id,
        }))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn ticketing_save_credentials(
    credentials: ticketing::TicketingCredentials,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use database::{Database, SettingsRepository, SettingsOps};

    // Get database path
    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    // Open database
    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = SettingsRepository::new(db.connection());

    // Save credentials
    repo.set("ticketing.api_key", &credentials.api_key).map_err(|e: rusqlite::Error| e.to_string())?;

    if let Some(team_id) = &credentials.team_id {
        repo.set("ticketing.team_id", team_id).map_err(|e: rusqlite::Error| e.to_string())?;
    }

    if let Some(workspace_id) = &credentials.workspace_id {
        repo.set("ticketing.workspace_id", workspace_id).map_err(|e: rusqlite::Error| e.to_string())?;
    }

    Ok(())
}

// Claude CLI commands

#[tauri::command]
fn get_claude_status() -> claude_cli::ClaudeStatus {
    claude_cli::get_claude_status()
}

#[tauri::command]
fn refresh_claude_status() -> claude_cli::ClaudeStatus {
    claude_cli::refresh_claude_status()
}

#[tauri::command]
async fn generate_bug_description(
    bug_context: claude_cli::BugContext,
) -> Result<claude_cli::ClaudeResponse, String> {
    use claude_cli::{PromptBuilder, PromptTask, ClaudeRequest, RealClaudeInvoker, ClaudeInvoker};

    // Check if Claude is ready
    let status = claude_cli::get_claude_status();
    if !status.is_ready() {
        return Err(format!("Claude CLI not ready: {:?}", status));
    }

    // Build prompt
    let prompt = PromptBuilder::build_prompt(
        &PromptTask::DescribeBug,
        Some(&bug_context),
        None,
    );

    // Create request with images
    let request = ClaudeRequest::new_with_images(
        prompt,
        bug_context.screenshot_paths.clone(),
        PromptTask::DescribeBug,
    )
    .with_bug_id(bug_context.bug_id.clone());

    // Invoke Claude
    let invoker = RealClaudeInvoker::new();
    invoker
        .invoke(request)
        .map_err(|e| format!("Failed to generate description: {}", e))
}

#[tauri::command]
async fn parse_console_screenshot(
    screenshot_path: String,
) -> Result<claude_cli::ClaudeResponse, String> {
    use claude_cli::{PromptBuilder, PromptTask, ClaudeRequest, RealClaudeInvoker, ClaudeInvoker};
    use std::path::PathBuf;

    // Check if Claude is ready
    let status = claude_cli::get_claude_status();
    if !status.is_ready() {
        return Err(format!("Claude CLI not ready: {:?}", status));
    }

    // Build prompt
    let prompt = PromptBuilder::build_console_parse_prompt();

    // Create request with the screenshot
    let request = ClaudeRequest::new_with_images(
        prompt,
        vec![PathBuf::from(screenshot_path)],
        PromptTask::ParseConsole,
    );

    // Invoke Claude
    let invoker = RealClaudeInvoker::new();
    invoker
        .invoke(request)
        .map_err(|e| format!("Failed to parse console: {}", e))
}

#[tauri::command]
async fn refine_bug_description(
    current_description: String,
    refinement_instructions: String,
    bug_id: String,
) -> Result<claude_cli::ClaudeResponse, String> {
    use claude_cli::{PromptBuilder, PromptTask, ClaudeRequest, RealClaudeInvoker, ClaudeInvoker};

    // Check if Claude is ready
    let status = claude_cli::get_claude_status();
    if !status.is_ready() {
        return Err(format!("Claude CLI not ready: {:?}", status));
    }

    // Build refinement prompt
    let prompt = PromptBuilder::build_refinement_prompt(
        &current_description,
        &refinement_instructions,
    );

    // Create request
    let request = ClaudeRequest::new_text(prompt, PromptTask::RefineDescription)
        .with_bug_id(bug_id);

    // Invoke Claude
    let invoker = RealClaudeInvoker::new();
    invoker
        .invoke(request)
        .map_err(|e| format!("Failed to refine description: {}", e))
}

#[tauri::command]
async fn save_bug_description(
    folder_path: String,
    description: String,
) -> Result<(), String> {
    use std::path::Path;

    // Ensure the folder exists
    let bug_folder = Path::new(&folder_path);
    if !bug_folder.exists() {
        return Err(format!("Bug folder does not exist: {}", folder_path));
    }

    // Write description to description.md file
    let description_file = bug_folder.join("description.md");
    std::fs::write(&description_file, description)
        .map_err(|e| format!("Failed to write description.md: {}", e))?;

    Ok(())
}

#[tauri::command]
fn update_bug_description(
    bug_id: String,
    description: String,
    app: tauri::AppHandle
) -> Result<(), String> {
    use database::{Database, BugOps, BugRepository};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = BugRepository::new(db.connection());

    let mut bug = repo.get(&bug_id)
        .map_err(|e: rusqlite::Error| e.to_string())?
        .ok_or_else(|| format!("Bug not found: {}", bug_id))?;

    bug.description = if description.is_empty() { None } else { Some(description) };

    repo.update(&bug)
        .map_err(|e: rusqlite::Error| e.to_string())
}

#[tauri::command]
fn format_session_export(session_folder_path: String) -> Result<(), String> {
    use std::path::Path;
    use std::fs;

    let session_path = Path::new(&session_folder_path);
    if !session_path.exists() {
        return Err(format!("Session folder does not exist: {}", session_folder_path));
    }

    // Read all entries in the session folder
    let entries = fs::read_dir(session_path)
        .map_err(|e| format!("Failed to read session folder: {}", e))?;

    // Collect bug folders (bug_XXX format) and sort them by bug number
    let mut bug_folders: Vec<(i32, String)> = Vec::new();

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(folder_name) = path.file_name().and_then(|n| n.to_str()) {
                // Check if folder name matches bug_XXX pattern
                if let Some(stripped) = folder_name.strip_prefix("bug_") {
                    if let Ok(bug_num) = stripped.parse::<i32>() {
                        bug_folders.push((bug_num, path.to_string_lossy().to_string()));
                    }
                }
            }
        }
    }

    // Sort by bug number
    bug_folders.sort_by_key(|(num, _)| *num);

    // Build the formatted output
    let mut output = String::new();

    for (i, (bug_num, bug_folder_path)) in bug_folders.iter().enumerate() {
        let bug_path = Path::new(bug_folder_path);
        let description_file = bug_path.join("description.md");

        // Read description.md if it exists
        let description = if description_file.exists() {
            fs::read_to_string(&description_file)
                .unwrap_or_else(|_| String::from("No description available."))
        } else {
            String::from("No description available.")
        };

        // Add bug header and description
        output.push_str(&format!("# Bug {:03}\n\n", bug_num));
        output.push_str(&description);

        // Add divider if not the last bug
        if i < bug_folders.len() - 1 {
            output.push_str("\n\n---\n\n");
        }
    }

    // Write to tickets-ready.md
    let tickets_ready_file = session_path.join("tickets-ready.md");
    fs::write(&tickets_ready_file, output)
        .map_err(|e| format!("Failed to write tickets-ready.md: {}", e))?;

    Ok(())
}

// ─── Settings Commands ───────────────────────────────────────────────────

#[tauri::command]
fn get_setting(key: String, app: tauri::AppHandle) -> Result<Option<String>, String> {
    use database::{Database, SettingsRepository, SettingsOps};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = SettingsRepository::new(db.connection());

    repo.get(&key).map_err(|e: rusqlite::Error| e.to_string())
}

#[tauri::command]
fn set_setting(key: String, value: String, app: tauri::AppHandle) -> Result<(), String> {
    use database::{Database, SettingsRepository, SettingsOps};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = SettingsRepository::new(db.connection());

    repo.set(&key, &value).map_err(|e: rusqlite::Error| e.to_string())
}

#[tauri::command]
fn get_all_settings(app: tauri::AppHandle) -> Result<Vec<database::Setting>, String> {
    use database::{Database, SettingsRepository, SettingsOps};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = SettingsRepository::new(db.connection());

    repo.get_all().map_err(|e: rusqlite::Error| e.to_string())
}

#[tauri::command]
fn delete_setting(key: String, app: tauri::AppHandle) -> Result<(), String> {
    use database::{Database, SettingsRepository, SettingsOps};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = SettingsRepository::new(db.connection());

    repo.delete(&key).map_err(|e: rusqlite::Error| e.to_string())
}

// ─── Setup Commands ──────────────────────────────────────────────────────

const SETUP_COMPLETE_KEY: &str = "has_completed_setup";

#[tauri::command]
fn has_completed_setup(app: tauri::AppHandle) -> Result<bool, String> {
    use database::{Database, SettingsRepository, SettingsOps};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = SettingsRepository::new(db.connection());

    match repo.get(SETUP_COMPLETE_KEY) {
        Ok(Some(value)) => Ok(value == "true"),
        Ok(None) => Ok(false),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
fn mark_setup_complete(app: tauri::AppHandle) -> Result<(), String> {
    use database::{Database, SettingsRepository, SettingsOps};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = SettingsRepository::new(db.connection());

    repo.set(SETUP_COMPLETE_KEY, "true")
        .map_err(|e: rusqlite::Error| e.to_string())
}

#[tauri::command]
fn reset_setup(app: tauri::AppHandle) -> Result<(), String> {
    use database::{Database, SettingsRepository, SettingsOps};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = SettingsRepository::new(db.connection());

    repo.delete(SETUP_COMPLETE_KEY)
        .map_err(|e: rusqlite::Error| e.to_string())
}

#[tauri::command]
fn get_bug_captures(bug_id: String, app: tauri::AppHandle) -> Result<Vec<database::Capture>, String> {
    use database::{Database, CaptureOps, CaptureRepository};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = CaptureRepository::new(db.connection());

    repo.list_by_bug(&bug_id)
        .map_err(|e: rusqlite::Error| e.to_string())
}

#[tauri::command]
fn get_unsorted_captures(session_id: String, app: tauri::AppHandle) -> Result<Vec<database::Capture>, String> {
    use database::{Database, CaptureOps, CaptureRepository};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = CaptureRepository::new(db.connection());

    repo.list_unsorted(&session_id)
        .map_err(|e: rusqlite::Error| e.to_string())
}

#[tauri::command]
fn assign_capture_to_bug(capture_id: String, bug_id: String, app: tauri::AppHandle) -> Result<(), String> {
    use database::{Database, CaptureOps, CaptureRepository};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = CaptureRepository::new(db.connection());

    let mut capture = repo.get(&capture_id)
        .map_err(|e: rusqlite::Error| e.to_string())?
        .ok_or_else(|| format!("Capture not found: {}", capture_id))?;

    capture.bug_id = Some(bug_id);

    repo.update(&capture)
        .map_err(|e: rusqlite::Error| e.to_string())
}

#[tauri::command]
fn get_app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

#[tauri::command]
fn enable_startup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use platform::{Platform, WindowsPlatform};
        let platform = WindowsPlatform;
        platform.enable_startup().map_err(|e| e.to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Startup configuration is only supported on Windows".to_string())
    }
}

#[tauri::command]
fn disable_startup() -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        use platform::{Platform, WindowsPlatform};
        let platform = WindowsPlatform;
        platform.disable_startup().map_err(|e| e.to_string())
    }

    #[cfg(not(target_os = "windows"))]
    {
        Err("Startup configuration is only supported on Windows".to_string())
    }
}

#[tauri::command]
fn update_bug_console_parse(
    bug_id: String,
    console_parsed_json: String,
    app: tauri::AppHandle
) -> Result<(), String> {
    use database::{Database, BugOps, BugRepository};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = BugRepository::new(db.connection());

    // Get the bug
    let mut bug = repo.get(&bug_id)
        .map_err(|e: rusqlite::Error| e.to_string())?
        .ok_or_else(|| format!("Bug not found: {}", bug_id))?;

    // Update the console_parse_json field
    bug.console_parse_json = Some(console_parsed_json);

    // Save back to database
    repo.update(&bug)
        .map_err(|e: rusqlite::Error| e.to_string())
}

#[tauri::command]
fn update_capture_console_flag(
    capture_id: String,
    is_console_capture: bool,
    app: tauri::AppHandle
) -> Result<(), String> {
    use database::{Database, CaptureOps, CaptureRepository};

    let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
        std::env::current_dir().unwrap().join("data")
    });
    let db_path = data_dir.join("qa_capture.db");

    let db = Database::open(&db_path).map_err(|e| e.to_string())?;
    let repo = CaptureRepository::new(db.connection());

    // Get the capture
    let mut capture = repo.get(&capture_id)
        .map_err(|e: rusqlite::Error| e.to_string())?
        .ok_or_else(|| format!("Capture not found: {}", capture_id))?;

    // Update the is_console_capture field
    capture.is_console_capture = is_console_capture;

    // Save back to database
    repo.update(&capture)
        .map_err(|e: rusqlite::Error| e.to_string())
}

// ─── Capture Bridge Commands ──────────────────────────────────────────

/// Trigger the OS screenshot tool (Snipping Tool on Windows).
/// Opens the snipping tool so the user can take a screenshot.
#[tauri::command]
fn trigger_screenshot() -> Result<(), String> {
    let bridge_guard = CAPTURE_BRIDGE.lock().unwrap();
    let bridge = bridge_guard
        .as_ref()
        .ok_or("Capture bridge not initialized")?;
    bridge.trigger_screenshot().map_err(|e| e.to_string())
}

/// Start watching the given folder for new screenshot/video files.
/// Emits `screenshot:captured` events to the frontend when files are detected.
/// Automatically stops any previously running watcher before starting a new one.
#[tauri::command]
fn start_file_watcher(folder_path: String, app: tauri::AppHandle) -> Result<(), String> {
    use std::sync::mpsc;
    use std::path::Path;

    let folder = Path::new(&folder_path);

    let (tx, rx) = mpsc::channel();

    let bridge_guard = CAPTURE_BRIDGE.lock().unwrap();
    let bridge = bridge_guard
        .as_ref()
        .ok_or("Capture bridge not initialized")?;

    // Stop any existing watcher first
    {
        let mut active = ACTIVE_WATCHER.lock().unwrap();
        if let Some(old_handle) = active.take() {
            let _ = bridge.stop_file_watcher(old_handle);
        }
    }

    let handle = bridge.start_file_watcher(folder, tx).map_err(|e| e.to_string())?;

    {
        let mut active = ACTIVE_WATCHER.lock().unwrap();
        *active = Some(handle);
    }

    // Spawn a background thread to forward capture events to the frontend
    let app_handle = app.clone();
    std::thread::spawn(move || {
        while let Ok(event) = rx.recv() {
            match event {
                platform::CaptureEvent::ScreenshotDetected { file_path, detected_at, .. } => {
                    let path_str = file_path.to_string_lossy().to_string();
                    let _ = app_handle.emit("screenshot:captured", serde_json::json!({
                        "filePath": path_str,
                        "timestamp": detected_at,
                    }));
                }
                platform::CaptureEvent::VideoDetected { file_path, detected_at, .. } => {
                    let path_str = file_path.to_string_lossy().to_string();
                    let _ = app_handle.emit("capture:file-detected", serde_json::json!({
                        "filePath": path_str,
                        "timestamp": detected_at,
                        "type": "video",
                    }));
                }
                platform::CaptureEvent::WatcherError { message } => {
                    eprintln!("File watcher error: {}", message);
                }
            }
        }
    });

    Ok(())
}

/// Stop the active file watcher (if any).
#[tauri::command]
fn stop_file_watcher() -> Result<(), String> {
    let bridge_guard = CAPTURE_BRIDGE.lock().unwrap();
    let bridge = bridge_guard
        .as_ref()
        .ok_or("Capture bridge not initialized")?;

    let mut active = ACTIVE_WATCHER.lock().unwrap();
    if let Some(handle) = active.take() {
        bridge.stop_file_watcher(handle).map_err(|e| e.to_string())
    } else {
        // No active watcher — not an error
        Ok(())
    }
}

// ─── Annotation Window Commands ──────────────────────────────────────

#[tauri::command]
async fn emit_screenshot_captured(
    file_path: String,
    app: tauri::AppHandle,
) -> Result<(), String> {
    // Emit screenshot:captured event to frontend
    app.emit("screenshot:captured", serde_json::json!({
        "filePath": file_path,
        "timestamp": chrono::Utc::now().timestamp_millis(),
    }))
    .map_err(|e| format!("Failed to emit screenshot:captured event: {}", e))
}

#[tauri::command]
async fn open_annotation_window(
    image_path: String,
    capture_id: Option<String>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    use std::path::Path;

    // Validate image path exists
    let path = Path::new(&image_path);
    if !path.exists() {
        return Err(format!("Image file not found: {}", image_path));
    }

    // Get primary monitor dimensions
    let monitor = app.primary_monitor()
        .map_err(|e| format!("Failed to get monitor info: {}", e))?
        .ok_or("No monitor found")?;

    let monitor_size = monitor.size();
    let monitor_width = monitor_size.width as f64;
    let monitor_height = monitor_size.height as f64;

    // Calculate 90% of viewport
    let max_width = monitor_width * 0.9;
    let max_height = monitor_height * 0.9;

    // For v1, we'll default to max size and let the component handle image scaling
    // In a production version, we'd read the image dimensions to calculate exact size
    let window_width = max_width.min(1200.0);
    let window_height = max_height.min(800.0);

    // Center the window
    let window_x = (monitor_width - window_width) / 2.0;
    let window_y = (monitor_height - window_height) / 2.0;

    // Create window ID based on image path to avoid duplicates
    let window_label = format!("annotation-{}",
        path.file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("window")
            .replace(".", "-"));

    // Check if window already exists
    if let Some(existing) = app.get_webview_window(&window_label) {
        existing.set_focus().map_err(|e| format!("Failed to focus existing window: {}", e))?;
        return Ok(());
    }

    // Build URL, optionally including capture_id for DB update after save
    let url = if let Some(cid) = capture_id {
        format!("/annotate?image={}&captureId={}", urlencoding::encode(&image_path), urlencoding::encode(&cid))
    } else {
        format!("/annotate?image={}", urlencoding::encode(&image_path))
    };

    tauri::WebviewWindowBuilder::new(
        &app,
        window_label,
        tauri::WebviewUrl::App(url.into())
    )
    .title("Annotate Screenshot")
    .inner_size(window_width, window_height)
    .position(window_x, window_y)
    .resizable(true)
    .decorations(false) // Frameless window per PRD: "Frameless or minimal frame. Always on top."
    .always_on_top(true)
    .focused(true)
    .build()
    .map_err(|e| format!("Failed to create annotation window: {}", e))?;

    Ok(())
}

/// Save an annotated screenshot from a base64-encoded PNG data URL.
///
/// `image_path` is the original screenshot path (used to derive the save path).
/// `data_url` is a data URL string like "data:image/png;base64,<base64data>".
/// `save_mode` is either "alongside" (default, saves as filename_annotated.png) or "overwrite".
/// `capture_id` is the optional DB capture ID — if provided, the annotated_path is stored in the DB.
///
/// Returns the path where the annotated file was written.
#[tauri::command]
fn save_annotated_image(
    image_path: String,
    data_url: String,
    save_mode: String,
    capture_id: Option<String>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    use std::path::Path;

    // Decode the data URL: strip the "data:image/png;base64," prefix
    let base64_data = data_url
        .split_once(',')
        .map(|x| x.1)
        .ok_or("Invalid data URL: missing comma separator")?;

    let image_bytes = base64::Engine::decode(
        &base64::engine::general_purpose::STANDARD,
        base64_data,
    )
    .map_err(|e| format!("Failed to decode base64 image data: {}", e))?;

    // Determine save path
    let original = Path::new(&image_path);
    let save_path = if save_mode == "overwrite" {
        image_path.clone()
    } else {
        // Save alongside original: e.g. screenshot.png -> screenshot_annotated.png
        let stem = original.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("screenshot");
        let ext = original.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("png");
        let parent = original.parent().unwrap_or(Path::new("."));
        parent.join(format!("{}_annotated.{}", stem, ext))
            .to_string_lossy()
            .to_string()
    };

    // Write the PNG bytes to disk
    std::fs::write(&save_path, &image_bytes)
        .map_err(|e| format!("Failed to write annotated image to {}: {}", save_path, e))?;

    // If a capture_id was provided, update the DB record
    if let Some(id) = capture_id {
        use database::{Database, CaptureOps, CaptureRepository};

        let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
            std::env::current_dir().unwrap().join("data")
        });
        let db_path = data_dir.join("qa_capture.db");

        let db = Database::open(&db_path).map_err(|e| e.to_string())?;
        let repo = CaptureRepository::new(db.connection());

        if let Ok(Some(mut capture)) = repo.get(&id) {
            capture.annotated_path = Some(save_path.clone());
            repo.update(&capture).map_err(|e: rusqlite::Error| e.to_string())?;
        }
    }

    Ok(save_path)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            // Initialize session manager
            let app_handle = app.handle().clone();
            let data_dir = app.path().app_data_dir().unwrap_or_else(|_| {
                std::env::current_dir().unwrap().join("data")
            });
            let db_path = data_dir.join("qa_capture.db");
            let storage_root = data_dir.join("sessions");

            // Create data directory if it doesn't exist
            std::fs::create_dir_all(&data_dir).ok();

            let emitter = Arc::new(TauriEventEmitter::new());
            emitter.set_app_handle(app_handle);

            let manager = Arc::new(SessionManager::new(
                db_path,
                storage_root,
                emitter as Arc<dyn EventEmitter>,
                Arc::new(RealFileSystem),
            ));

            *SESSION_MANAGER.lock().unwrap() = Some(manager);

            // Initialize capture bridge (platform-specific screenshot/file-watcher)
            *CAPTURE_BRIDGE.lock().unwrap() = Some(platform::get_capture_bridge());

            // Initialize hotkey manager and load config from settings
            let hotkey_manager = Arc::new(HotkeyManager::new());

            // Load hotkey config from database settings
            let app_handle_for_settings = app.handle().clone();
            let loaded_config = hotkey_manager.load_from_settings(|key| {
                use database::{Database, SettingsRepository, SettingsOps};
                let data_dir = app_handle_for_settings.path().app_data_dir().unwrap_or_else(|_| {
                    std::env::current_dir().unwrap().join("data")
                });
                let db_path = data_dir.join("qa_capture.db");

                if let Ok(db) = Database::open(&db_path) {
                    let repo = SettingsRepository::new(db.connection());
                    repo.get(key).ok().flatten()
                } else {
                    None
                }
            });

            // Update the manager's config with loaded settings and register hotkeys
            // update_config() already calls register_all() internally, so no separate call needed
            let registration_results = hotkey_manager.update_config(app.handle(), loaded_config);

            // Check for registration failures and notify via tray
            let mut failed_shortcuts = Vec::new();
            for result in registration_results {
                if let Err(e) = result {
                    eprintln!("Hotkey registration error: {}", e);
                    failed_shortcuts.push(e);
                }
            }

            // If any hotkeys failed to register, show a notification via tray tooltip
            if !failed_shortcuts.is_empty() {
                let error_count = failed_shortcuts.len();
                eprintln!(
                    "Warning: {} hotkey(s) failed to register. Check logs for details.",
                    error_count
                );
                // The tray will be built next, and we'll update its tooltip after it's created
            }

            *HOTKEY_MANAGER.lock().unwrap() = Some(hotkey_manager);

            // Initialize ticketing integration (Linear by default)
            let ticketing_integration: Arc<dyn TicketingIntegration> = Arc::new(LinearIntegration::new());
            *TICKETING_INTEGRATION.lock().unwrap() = Some(ticketing_integration);

            // Build tray menu
            let menu = Menu::new(app)?;
            let toggle_item = MenuItemBuilder::new("Start Session")
                .id("start-session")
                .enabled(true)
                .build(app)?;
            let capture_item = MenuItemBuilder::new("New Bug Capture")
                .id("new-bug-capture")
                .enabled(true)
                .build(app)?;
            let show_item = MenuItemBuilder::new("Open Main Window")
                .id("open-main-window")
                .enabled(true)
                .build(app)?;
            let settings_item = MenuItemBuilder::new("Settings")
                .id("settings")
                .enabled(true)
                .build(app)?;
            let help_item = MenuItemBuilder::new("Help / User Guide")
                .id("help")
                .enabled(true)
                .build(app)?;
            let quit_item = MenuItemBuilder::new("Quit")
                .id("quit")
                .enabled(true)
                .build(app)?;

            menu.append(&toggle_item)?;
            menu.append(&capture_item)?;
            menu.append(&show_item)?;
            menu.append(&settings_item)?;
            menu.append(&help_item)?;
            menu.append(&quit_item)?;

            // Build tray icon and store in static to prevent it from being dropped
            let tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("Unbroken QA Capture - Idle")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app_handle, event| {
                    match event.id().as_ref() {
                        "start-session" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                            app_handle.emit("tray-menu-start-session", ()).ok();
                        }
                        "new-bug-capture" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                            app_handle.emit("tray-menu-new-bug", ()).ok();
                        }
                        "open-main-window" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                            app_handle.emit("tray-window-shown", ()).ok();
                        }
                        "settings" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                            app_handle.emit("tray-menu-settings", ()).ok();
                        }
                        "end-session" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                            app_handle.emit("tray-menu-end-session", ()).ok();
                        }
                        "end-bug-capture" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                            app_handle.emit("tray-menu-end-bug-capture", ()).ok();
                        }
                        "open-review" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                            app_handle.emit("tray-menu-open-review", ()).ok();
                        }
                        "help" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                            app_handle.emit("tray-menu-help", ()).ok();
                        }
                        "quit" => {
                            app_handle.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        let app_handle = tray.app_handle();
                        if let Some(window) = app_handle.get_webview_window("main") {
                            window.show().ok();
                            window.set_focus().ok();
                        }
                        app_handle.emit("tray-window-shown", ()).ok();
                    }
                })
                .build(app)?;

            *TRAY_ICON.lock().unwrap() = Some(tray);

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            set_custom_template_path,
            render_bug_template,
            reload_template,
            get_template_source,
            save_custom_template,
            reset_template_to_default,
            get_template_path,
            open_template_in_editor,
            copy_bug_to_clipboard,
            open_bug_folder,
            open_session_folder,
            update_tray_icon,
            update_tray_menu,
            update_tray_tooltip,
            get_bug_notes,
            update_bug_notes,
            get_session_notes,
            update_session_notes,
            open_session_notes_window,
            start_session,
            end_session,
            resume_session,
            start_bug_capture,
            end_bug_capture,
            resume_bug_capture,
            get_active_session_id,
            get_active_bug_id,
            get_active_session,
            list_sessions,
            update_session_status,
            get_bugs_by_session,
            get_session_summaries,
            generate_session_summary,
            get_hotkey_config,
            update_hotkey_config,
            is_hotkey_registered,
            ticketing_authenticate,
            ticketing_create_ticket,
            ticketing_check_connection,
            ticketing_get_credentials,
            ticketing_save_credentials,
            get_claude_status,
            refresh_claude_status,
            generate_bug_description,
            parse_console_screenshot,
            refine_bug_description,
            save_bug_description,
            format_session_export,
            get_setting,
            set_setting,
            get_all_settings,
            delete_setting,
            has_completed_setup,
            mark_setup_complete,
            reset_setup,
            get_bug_captures,
            get_unsorted_captures,
            assign_capture_to_bug,
            update_bug_console_parse,
            update_bug_description,
            update_capture_console_flag,
            get_app_version,
            enable_startup,
            disable_startup,
            emit_screenshot_captured,
            open_annotation_window,
            save_annotated_image,
            trigger_screenshot,
            start_file_watcher,
            stop_file_watcher
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                // Only intercept the main window — other windows (session notes, annotation)
                // should close normally.
                if window.label() != "main" {
                    return;
                }
                // Instead of closing the app, hide the window to system tray
                window.hide().unwrap();
                api.prevent_close();
            }
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_bug_folder(temp_dir: &std::path::Path) -> String {
        let bug_folder = temp_dir.join("test_bug");
        std::fs::create_dir_all(&bug_folder).unwrap();

        let bug_data = template::BugData {
            title: "Test Bug".to_string(),
            bug_type: "UI".to_string(),
            description_steps: "1. Click button\n2. Observe error".to_string(),
            description_expected: "Button should work".to_string(),
            description_actual: "Button crashes app".to_string(),
            metadata: template::BugMetadata {
                meeting_id: Some("MTG-123".to_string()),
                software_version: Some("1.0.0".to_string()),
                environment: template::Environment {
                    os: "Windows 11".to_string(),
                    display_resolution: "1920x1080".to_string(),
                    dpi_scaling: "100%".to_string(),
                    ram: "16GB".to_string(),
                    cpu: "Intel i7".to_string(),
                    foreground_app: "TestApp".to_string(),
                },
                console_captures: vec![],
                custom_fields: HashMap::new(),
            },
            folder_path: bug_folder.to_string_lossy().to_string(),
            captures: vec!["screenshot1.png".to_string()],
            console_output: Some("Error: Something went wrong".to_string()),
        };

        let metadata_path = bug_folder.join("metadata.json");
        let json = serde_json::to_string_pretty(&bug_data).unwrap();
        std::fs::write(&metadata_path, json).unwrap();

        bug_folder.to_string_lossy().to_string()
    }

    #[test]
    fn test_read_and_render_bug_success() {
        let temp_dir = std::env::temp_dir().join("test_copy_bug");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let bug_folder = create_test_bug_folder(&temp_dir);
        let result = read_and_render_bug(&bug_folder);

        assert!(result.is_ok());
        let rendered = result.unwrap();

        // Verify the rendered output contains key information
        assert!(rendered.contains("Test Bug"));
        assert!(rendered.contains("UI"));
        assert!(rendered.contains("Click button"));
        assert!(rendered.contains("Windows 11"));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_read_and_render_bug_folder_not_found() {
        let result = read_and_render_bug("/nonexistent/folder/path");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Bug folder does not exist"));
    }

    #[test]
    fn test_read_and_render_bug_missing_metadata() {
        let temp_dir = std::env::temp_dir().join("test_copy_bug_no_metadata");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let bug_folder = temp_dir.join("test_bug");
        std::fs::create_dir_all(&bug_folder).unwrap();

        let result = read_and_render_bug(&bug_folder.to_string_lossy());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("metadata.json not found"));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_read_and_render_bug_invalid_json() {
        let temp_dir = std::env::temp_dir().join("test_copy_bug_invalid");
        std::fs::create_dir_all(&temp_dir).unwrap();

        let bug_folder = temp_dir.join("test_bug");
        std::fs::create_dir_all(&bug_folder).unwrap();

        let metadata_path = bug_folder.join("metadata.json");
        std::fs::write(&metadata_path, "invalid json content").unwrap();

        let result = read_and_render_bug(&bug_folder.to_string_lossy());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to parse metadata.json"));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_folder_validation_for_open_commands() {
        use std::path::Path;

        // Test nonexistent folder
        let nonexistent = "/nonexistent/folder/path";
        let path = Path::new(nonexistent);
        assert!(!path.exists());

        // Test valid folder
        let temp_dir = std::env::temp_dir().join("test_open_folder");
        std::fs::create_dir_all(&temp_dir).unwrap();
        assert!(temp_dir.exists());
        assert!(temp_dir.is_dir());

        // Test file (not a directory)
        let test_file = temp_dir.join("test_file.txt");
        std::fs::write(&test_file, "test content").unwrap();
        assert!(test_file.exists());
        assert!(!test_file.is_dir());

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_setup_tracking() {
        use database::{Database, SettingsRepository, SettingsOps};

        // Create in-memory database
        let db = Database::in_memory().unwrap();
        let repo = SettingsRepository::new(db.connection());

        // Initially, setup should not be completed
        let has_setup = repo.get(SETUP_COMPLETE_KEY).unwrap();
        assert!(has_setup.is_none());

        // Mark setup as complete
        repo.set(SETUP_COMPLETE_KEY, "true").unwrap();

        // Verify setup is marked as complete
        let has_setup = repo.get(SETUP_COMPLETE_KEY).unwrap();
        assert_eq!(has_setup, Some("true".to_string()));

        // Reset setup
        repo.delete(SETUP_COMPLETE_KEY).unwrap();

        // Verify setup is reset
        let has_setup = repo.get(SETUP_COMPLETE_KEY).unwrap();
        assert!(has_setup.is_none());
    }

    #[test]
    fn test_setup_complete_flag_parsing() {
        use database::{Database, SettingsRepository, SettingsOps};

        let db = Database::in_memory().unwrap();
        let repo = SettingsRepository::new(db.connection());

        // Test "true" value
        repo.set(SETUP_COMPLETE_KEY, "true").unwrap();
        let value = repo.get(SETUP_COMPLETE_KEY).unwrap();
        assert_eq!(value, Some("true".to_string()));

        // Test "false" value
        repo.set(SETUP_COMPLETE_KEY, "false").unwrap();
        let value = repo.get(SETUP_COMPLETE_KEY).unwrap();
        assert_eq!(value, Some("false".to_string()));

        // Test empty value
        repo.set(SETUP_COMPLETE_KEY, "").unwrap();
        let value = repo.get(SETUP_COMPLETE_KEY).unwrap();
        assert_eq!(value, Some("".to_string()));
    }

    #[test]
    fn test_setup_persistence() {
        use database::{Database, SettingsRepository, SettingsOps};

        let temp_dir = std::env::temp_dir().join("test_setup_persistence");
        std::fs::create_dir_all(&temp_dir).unwrap();
        let db_path = temp_dir.join("test.db");

        // Create database and mark setup complete
        {
            let db = Database::open(&db_path).unwrap();
            let repo = SettingsRepository::new(db.connection());
            repo.set(SETUP_COMPLETE_KEY, "true").unwrap();
        }

        // Reopen database and verify setup is still complete
        {
            let db = Database::open(&db_path).unwrap();
            let repo = SettingsRepository::new(db.connection());
            let value = repo.get(SETUP_COMPLETE_KEY).unwrap();
            assert_eq!(value, Some("true".to_string()));
        }

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_format_session_export_with_bugs() {
        let temp_dir = std::env::temp_dir().join("test_session_export");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create bug folders with description.md files
        let bug1_folder = temp_dir.join("bug_001");
        std::fs::create_dir_all(&bug1_folder).unwrap();
        std::fs::write(
            bug1_folder.join("description.md"),
            "This is the first bug description."
        ).unwrap();

        let bug2_folder = temp_dir.join("bug_002");
        std::fs::create_dir_all(&bug2_folder).unwrap();
        std::fs::write(
            bug2_folder.join("description.md"),
            "This is the second bug description."
        ).unwrap();

        let bug3_folder = temp_dir.join("bug_003");
        std::fs::create_dir_all(&bug3_folder).unwrap();
        std::fs::write(
            bug3_folder.join("description.md"),
            "This is the third bug description."
        ).unwrap();

        // Call format_session_export
        let result = format_session_export(temp_dir.to_string_lossy().to_string());
        assert!(result.is_ok());

        // Read and verify tickets-ready.md
        let tickets_file = temp_dir.join("tickets-ready.md");
        assert!(tickets_file.exists());

        let content = std::fs::read_to_string(&tickets_file).unwrap();

        // Verify all bugs are present
        assert!(content.contains("# Bug 001"));
        assert!(content.contains("This is the first bug description."));
        assert!(content.contains("# Bug 002"));
        assert!(content.contains("This is the second bug description."));
        assert!(content.contains("# Bug 003"));
        assert!(content.contains("This is the third bug description."));

        // Verify dividers are present (2 dividers for 3 bugs)
        assert_eq!(content.matches("---").count(), 2);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_format_session_export_empty_session() {
        let temp_dir = std::env::temp_dir().join("test_session_export_empty");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Call format_session_export on empty session folder
        let result = format_session_export(temp_dir.to_string_lossy().to_string());
        assert!(result.is_ok());

        // Read and verify tickets-ready.md exists but is empty
        let tickets_file = temp_dir.join("tickets-ready.md");
        assert!(tickets_file.exists());

        let content = std::fs::read_to_string(&tickets_file).unwrap();
        assert_eq!(content, "");

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_format_session_export_missing_description() {
        let temp_dir = std::env::temp_dir().join("test_session_export_missing_desc");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create bug folder without description.md
        let bug1_folder = temp_dir.join("bug_001");
        std::fs::create_dir_all(&bug1_folder).unwrap();

        // Call format_session_export
        let result = format_session_export(temp_dir.to_string_lossy().to_string());
        assert!(result.is_ok());

        // Read and verify tickets-ready.md
        let tickets_file = temp_dir.join("tickets-ready.md");
        assert!(tickets_file.exists());

        let content = std::fs::read_to_string(&tickets_file).unwrap();

        // Verify placeholder text is used
        assert!(content.contains("# Bug 001"));
        assert!(content.contains("No description available."));

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_format_session_export_nonexistent_folder() {
        let result = format_session_export("/nonexistent/folder/path".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Session folder does not exist"));
    }

    #[test]
    fn test_format_session_export_bug_numbering_order() {
        let temp_dir = std::env::temp_dir().join("test_session_export_order");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create bugs out of order
        let bug3_folder = temp_dir.join("bug_003");
        std::fs::create_dir_all(&bug3_folder).unwrap();
        std::fs::write(bug3_folder.join("description.md"), "Bug 3").unwrap();

        let bug1_folder = temp_dir.join("bug_001");
        std::fs::create_dir_all(&bug1_folder).unwrap();
        std::fs::write(bug1_folder.join("description.md"), "Bug 1").unwrap();

        let bug2_folder = temp_dir.join("bug_002");
        std::fs::create_dir_all(&bug2_folder).unwrap();
        std::fs::write(bug2_folder.join("description.md"), "Bug 2").unwrap();

        // Call format_session_export
        let result = format_session_export(temp_dir.to_string_lossy().to_string());
        assert!(result.is_ok());

        // Read tickets-ready.md
        let tickets_file = temp_dir.join("tickets-ready.md");
        let content = std::fs::read_to_string(&tickets_file).unwrap();

        // Verify bugs appear in correct order
        let bug1_pos = content.find("Bug 1").unwrap();
        let bug2_pos = content.find("Bug 2").unwrap();
        let bug3_pos = content.find("Bug 3").unwrap();

        assert!(bug1_pos < bug2_pos);
        assert!(bug2_pos < bug3_pos);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_format_session_export_mixed_folders() {
        let temp_dir = std::env::temp_dir().join("test_session_export_mixed");
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Create bug folders
        let bug1_folder = temp_dir.join("bug_001");
        std::fs::create_dir_all(&bug1_folder).unwrap();
        std::fs::write(bug1_folder.join("description.md"), "Bug 1").unwrap();

        // Create other folders that should be ignored
        let other_folder = temp_dir.join("session-notes");
        std::fs::create_dir_all(&other_folder).unwrap();
        std::fs::write(other_folder.join("notes.md"), "Notes").unwrap();

        let captures_folder = temp_dir.join("_captures");
        std::fs::create_dir_all(&captures_folder).unwrap();

        // Create a file in session root (should be ignored)
        std::fs::write(temp_dir.join("session-notes.md"), "Session notes").unwrap();

        // Call format_session_export
        let result = format_session_export(temp_dir.to_string_lossy().to_string());
        assert!(result.is_ok());

        // Read tickets-ready.md
        let tickets_file = temp_dir.join("tickets-ready.md");
        let content = std::fs::read_to_string(&tickets_file).unwrap();

        // Verify only bug folders are included
        assert!(content.contains("# Bug 001"));
        assert!(content.contains("Bug 1"));
        assert!(!content.contains("Notes"));
        assert_eq!(content.matches("# Bug").count(), 1);

        // Cleanup
        std::fs::remove_dir_all(&temp_dir).ok();
    }

    // ------------------------------------------------------------------
    // Tray icon tests
    // ------------------------------------------------------------------

    #[test]
    fn tray_icon_decodes_successfully_for_all_states() {
        // Verify that each state loads a valid, decodable 32x32 RGBA icon.
        for state in &["idle", "active", "bug", "review"] {
            let result = tray_icon_for_state(state);
            assert!(
                result.is_ok(),
                "tray_icon_for_state('{}') returned error: {:?}",
                state,
                result.err()
            );
            let icon = result.unwrap();
            // 32x32 RGBA = 4096 bytes
            assert_eq!(icon.width(), 32, "Icon for '{}' should be 32px wide", state);
            assert_eq!(icon.height(), 32, "Icon for '{}' should be 32px tall", state);
            assert_eq!(
                icon.rgba().len(),
                32 * 32 * 4,
                "Icon for '{}' should have 32*32*4 RGBA bytes",
                state
            );
        }
    }

    #[test]
    fn tray_icon_unknown_state_falls_back_to_idle() {
        // Unknown states should use the idle icon without panicking.
        let result = tray_icon_for_state("unknown-state");
        assert!(result.is_ok(), "tray_icon_for_state('unknown-state') should fall back to idle");
        let icon = result.unwrap();
        assert_eq!(icon.width(), 32);
        assert_eq!(icon.height(), 32);
    }

    #[test]
    fn tray_icon_states_have_distinct_colors() {
        // Each state icon should have a visually distinct dominant color.
        // We sample the center pixel (16,16) of each 32x32 icon.
        let states_and_expected_channel = [
            // (state, dominant_channel_index): 0=R, 1=G, 2=B
            // idle: gray (R≈G≈B≈128), all channels roughly equal
            // active: green — G channel should be highest
            ("active", 1usize), // green channel
            ("bug",    0usize), // red channel
            ("review", 2usize), // blue channel
        ];

        for (state, dominant) in &states_and_expected_channel {
            let icon = tray_icon_for_state(state).unwrap();
            // Center pixel of 32x32 is at row 15, col 15
            let idx = (15 * 32 + 15) * 4;
            let rgba = icon.rgba();
            let r = rgba[idx] as u32;
            let g = rgba[idx + 1] as u32;
            let b = rgba[idx + 2] as u32;
            let channels = [r, g, b];
            let max_ch = channels.iter().enumerate().max_by_key(|(_, &v)| v).unwrap().0;
            assert_eq!(
                max_ch, *dominant,
                "State '{}': expected channel {} to dominate, got R={} G={} B={}",
                state, dominant, r, g, b
            );
        }
    }

    #[test]
    fn decode_png_rgba_handles_valid_png() {
        // Decode a known-good embedded PNG and verify dimensions.
        let png_bytes = include_bytes!("../icons/tray/tray-idle-32.png");
        let result = decode_png_rgba(png_bytes);
        assert!(result.is_ok(), "decode_png_rgba failed: {:?}", result.err());
        let (rgba, w, h) = result.unwrap();
        assert_eq!(w, 32);
        assert_eq!(h, 32);
        assert_eq!(rgba.len(), 32 * 32 * 4);
    }

    // ------------------------------------------------------------------
    // Capture naming convention tests (PRD §10)
    // ------------------------------------------------------------------

    #[test]
    fn test_make_capture_filename_screenshot() {
        use database::CaptureType;
        let path = std::path::Path::new("screenshot_20240217_143025.png");
        let (name, ctype) = make_capture_filename(path, 1);
        assert_eq!(name, "capture-001.png");
        assert_eq!(ctype, CaptureType::Screenshot);

        let (name2, _) = make_capture_filename(path, 42);
        assert_eq!(name2, "capture-042.png");
    }

    #[test]
    fn test_make_capture_filename_video_mp4() {
        use database::CaptureType;
        let path = std::path::Path::new("recording.mp4");
        let (name, ctype) = make_capture_filename(path, 1);
        assert_eq!(name, "recording-001.mp4");
        assert_eq!(ctype, CaptureType::Video);
    }

    #[test]
    fn test_make_capture_filename_video_webm() {
        use database::CaptureType;
        let path = std::path::Path::new("clip.webm");
        let (name, ctype) = make_capture_filename(path, 5);
        assert_eq!(name, "recording-005.webm");
        assert_eq!(ctype, CaptureType::Video);
    }

    #[test]
    fn test_make_capture_filename_jpg() {
        use database::CaptureType;
        let path = std::path::Path::new("image.jpg");
        let (name, ctype) = make_capture_filename(path, 99);
        assert_eq!(name, "capture-099.jpg");
        assert_eq!(ctype, CaptureType::Screenshot);
    }

    #[test]
    fn test_next_capture_number_empty_dir() {
        let temp_dir = std::env::temp_dir().join(format!("test_capture_num_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Empty dir — next number should be 1
        assert_eq!(next_capture_number(&temp_dir), 1);

        std::fs::remove_dir_all(&temp_dir).ok();
    }

    #[test]
    fn test_next_capture_number_with_existing_captures() {
        let temp_dir = std::env::temp_dir().join(format!("test_capture_num_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&temp_dir).unwrap();

        // Add some capture files
        std::fs::write(temp_dir.join("capture-001.png"), "").unwrap();
        std::fs::write(temp_dir.join("capture-002.png"), "").unwrap();
        std::fs::write(temp_dir.join("recording-003.mp4"), "").unwrap();
        // Non-capture file should not count
        std::fs::write(temp_dir.join("notes.md"), "").unwrap();

        assert_eq!(next_capture_number(&temp_dir), 4);

        std::fs::remove_dir_all(&temp_dir).ok();
    }
}
