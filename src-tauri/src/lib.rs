mod template;
mod database;
pub mod platform;
mod session_manager;
mod hotkey;

#[cfg(test)]
mod hotkey_tests;

use std::sync::{Arc, Mutex};
use template::TemplateManager;
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{TrayIconBuilder, TrayIconEvent};
use tauri::{Manager, Emitter, AppHandle};
use session_manager::{SessionManager, EventEmitter, RealFileSystem};
use hotkey::{HotkeyManager, HotkeyConfig};

// Global template manager
static TEMPLATE_MANAGER: Mutex<Option<TemplateManager>> = Mutex::new(None);

// Global session manager
static SESSION_MANAGER: Mutex<Option<Arc<SessionManager>>> = Mutex::new(None);

// Global hotkey manager
static HOTKEY_MANAGER: Mutex<Option<Arc<HotkeyManager>>> = Mutex::new(None);

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

#[tauri::command]
async fn update_tray_icon(state: String, app_handle: tauri::AppHandle) -> Result<(), String> {
    // Emit event to update tray icon based on state
    // States: idle, active, bug, review
    app_handle
        .emit("tray-state-changed", state)
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

// ─── Session Manager Commands ────────────────────────────────────────────

#[tauri::command]
fn start_session() -> Result<database::Session, String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Session manager not initialized")?;
    manager.start_session()
}

#[tauri::command]
fn end_session(session_id: String) -> Result<(), String> {
    let manager_guard = SESSION_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Session manager not initialized")?;
    manager.end_session(&session_id)
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
    let manager_guard = HOTKEY_MANAGER.lock().unwrap();
    let manager = manager_guard
        .as_ref()
        .ok_or("Hotkey manager not initialized")?;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
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

            // Initialize hotkey manager
            let hotkey_manager = Arc::new(HotkeyManager::new());
            let registration_results = hotkey_manager.register_all(app.handle());

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

            // Build tray menu
            let menu = Menu::new(app)?;
            let toggle_item = MenuItem::new(app, "Start Session", true, None::<&str>)?;
            let capture_item = MenuItem::new(app, "New Bug Capture", true, None::<&str>)?;
            let show_item = MenuItem::new(app, "Open Main Window", true, None::<&str>)?;
            let settings_item = MenuItem::new(app, "Settings", true, None::<&str>)?;
            let quit_item = MenuItem::new(app, "Quit", true, None::<&str>)?;

            menu.append(&toggle_item)?;
            menu.append(&capture_item)?;
            menu.append(&show_item)?;
            menu.append(&settings_item)?;
            menu.append(&quit_item)?;

            // Build tray icon
            let _tray = TrayIconBuilder::with_id("main-tray")
                .tooltip("Unbroken QA Capture - Idle")
                .icon(app.default_window_icon().unwrap().clone())
                .menu(&menu)
                .on_menu_event(|app_handle, event| {
                    match event.id().as_ref() {
                        "Start Session" => {
                            app_handle.emit("tray-menu-start-session", ()).ok();
                        }
                        "New Bug Capture" => {
                            app_handle.emit("tray-menu-new-bug", ()).ok();
                        }
                        "Open Main Window" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                        }
                        "Settings" => {
                            app_handle.emit("tray-menu-settings", ()).ok();
                        }
                        "Quit" => {
                            app_handle.exit(0);
                        }
                        _ => {}
                    }
                })
                .on_tray_icon_event(|tray, event| {
                    if let TrayIconEvent::Click { button: tauri::tray::MouseButton::Left, .. } = event {
                        if let Some(app) = tray.app_handle().get_webview_window("main") {
                            app.show().ok();
                            app.set_focus().ok();
                        }
                    }
                })
                .build(app)?;

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            greet,
            set_custom_template_path,
            render_bug_template,
            reload_template,
            copy_bug_to_clipboard,
            open_bug_folder,
            open_session_folder,
            update_tray_icon,
            update_tray_tooltip,
            get_bug_notes,
            update_bug_notes,
            get_session_notes,
            update_session_notes,
            start_session,
            end_session,
            resume_session,
            start_bug_capture,
            end_bug_capture,
            get_active_session_id,
            get_active_bug_id,
            get_hotkey_config,
            update_hotkey_config,
            is_hotkey_registered
        ])
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
}
