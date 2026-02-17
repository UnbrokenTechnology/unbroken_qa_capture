mod template;
mod database;
pub mod platform;
mod session_manager;
mod session_summary;
mod hotkey;
mod claude_cli;
mod ticketing;

#[cfg(test)]
mod hotkey_tests;

use std::sync::{Arc, Mutex};
use template::TemplateManager;
use tauri_plugin_clipboard_manager::ClipboardExt;
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

    // Create annotation window
    let url = format!("/annotate?image={}", urlencoding::encode(&image_path));

    tauri::WebviewWindowBuilder::new(
        &app,
        window_label,
        tauri::WebviewUrl::App(url.into())
    )
    .title("Annotate Screenshot")
    .inner_size(window_width, window_height)
    .position(window_x, window_y)
    .resizable(true)
    .decorations(true) // Use system decorations for v1, can be minimized for v2
    .always_on_top(true)
    .focused(true)
    .build()
    .map_err(|e| format!("Failed to create annotation window: {}", e))?;

    Ok(())
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
            let quit_item = MenuItemBuilder::new("Quit")
                .id("quit")
                .enabled(true)
                .build(app)?;

            menu.append(&toggle_item)?;
            menu.append(&capture_item)?;
            menu.append(&show_item)?;
            menu.append(&settings_item)?;
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
                        }
                        "settings" => {
                            if let Some(window) = app_handle.get_webview_window("main") {
                                window.show().ok();
                                window.set_focus().ok();
                            }
                            app_handle.emit("tray-menu-settings", ()).ok();
                        }
                        "quit" => {
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
            update_capture_console_flag,
            get_app_version,
            enable_startup,
            disable_startup,
            emit_screenshot_captured,
            open_annotation_window
        ])
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
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
}
