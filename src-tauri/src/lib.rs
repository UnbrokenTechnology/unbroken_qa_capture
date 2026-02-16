mod template;

use std::sync::Mutex;
use template::TemplateManager;

// Global template manager
static TEMPLATE_MANAGER: Mutex<Option<TemplateManager>> = Mutex::new(None);

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            set_custom_template_path,
            render_bug_template,
            reload_template
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
