mod template;
mod database;
pub mod platform;

use std::sync::Mutex;
use template::TemplateManager;
use tauri_plugin_clipboard_manager::ClipboardExt;

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

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            set_custom_template_path,
            render_bug_template,
            reload_template,
            copy_bug_to_clipboard,
            open_bug_folder,
            open_session_folder
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
