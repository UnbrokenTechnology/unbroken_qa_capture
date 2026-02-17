use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Emitter};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

/// Represents a hotkey action that can be triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HotkeyAction {
    ToggleSession,
    StartBugCapture,
    EndBugCapture,
    OpenQuickNotepad,
    OpenSessionNotepad,
}

impl HotkeyAction {
    /// Get the event name that should be emitted when this action is triggered
    pub fn event_name(&self) -> &'static str {
        match self {
            HotkeyAction::ToggleSession => "hotkey-toggle-session",
            HotkeyAction::StartBugCapture => "hotkey-start-bug-capture",
            HotkeyAction::EndBugCapture => "hotkey-end-bug-capture",
            HotkeyAction::OpenQuickNotepad => "hotkey-open-quick-notepad",
            HotkeyAction::OpenSessionNotepad => "hotkey-open-session-notepad",
        }
    }

    /// Get a human-readable description of this action
    pub fn description(&self) -> &'static str {
        match self {
            HotkeyAction::ToggleSession => "Toggle Session",
            HotkeyAction::StartBugCapture => "Start Bug Capture",
            HotkeyAction::EndBugCapture => "End Bug Capture",
            HotkeyAction::OpenQuickNotepad => "Open Quick Notepad",
            HotkeyAction::OpenSessionNotepad => "Open Session Notepad",
        }
    }
}

/// Configuration for hotkeys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotkeyConfig {
    pub shortcuts: HashMap<HotkeyAction, String>,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        let mut shortcuts = HashMap::new();
        shortcuts.insert(HotkeyAction::ToggleSession, "Ctrl+Shift+Q".to_string());
        shortcuts.insert(HotkeyAction::StartBugCapture, "PrintScreen".to_string());
        shortcuts.insert(HotkeyAction::EndBugCapture, "F4".to_string());
        shortcuts.insert(
            HotkeyAction::OpenQuickNotepad,
            "Ctrl+Shift+N".to_string(),
        );
        shortcuts.insert(
            HotkeyAction::OpenSessionNotepad,
            "Ctrl+Shift+M".to_string(),
        );
        Self { shortcuts }
    }
}

/// Manages global hotkey registration and handling
pub struct HotkeyManager {
    config: Arc<Mutex<HotkeyConfig>>,
    registered_shortcuts: Arc<Mutex<Vec<String>>>,
}

impl HotkeyManager {
    /// Create a new hotkey manager with default configuration
    pub fn new() -> Self {
        Self {
            config: Arc::new(Mutex::new(HotkeyConfig::default())),
            registered_shortcuts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create a new hotkey manager with custom configuration
    #[allow(dead_code)]
    pub fn with_config(config: HotkeyConfig) -> Self {
        Self {
            config: Arc::new(Mutex::new(config)),
            registered_shortcuts: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register all configured hotkeys with the application
    pub fn register_all(&self, app: &AppHandle) -> Vec<Result<(), String>> {
        let config = self.config.lock().unwrap();
        let mut results = Vec::new();
        let mut registered = Vec::new();

        for (action, shortcut_str) in &config.shortcuts {
            match self.register_hotkey(app, action, shortcut_str) {
                Ok(_) => {
                    registered.push(shortcut_str.clone());
                    results.push(Ok(()));
                }
                Err(e) => {
                    results.push(Err(format!(
                        "Failed to register '{}' for {}: {}",
                        shortcut_str,
                        action.description(),
                        e
                    )));
                }
            }
        }

        *self.registered_shortcuts.lock().unwrap() = registered;
        results
    }

    /// Register a single hotkey
    fn register_hotkey(
        &self,
        app: &AppHandle,
        action: &HotkeyAction,
        shortcut_str: &str,
    ) -> Result<(), String> {
        let shortcut: Shortcut = shortcut_str
            .parse()
            .map_err(|e| format!("Invalid shortcut format: {}", e))?;

        let event_name = action.event_name().to_string();
        let app_clone = app.clone();

        app.global_shortcut()
            .on_shortcut(shortcut, move |_app, _shortcut, _event| {
                app_clone.emit(&event_name, ()).ok();
            })
            .map_err(|e| format!("Failed to register shortcut: {}", e))?;

        Ok(())
    }

    /// Unregister all hotkeys
    pub fn unregister_all(&self, app: &AppHandle) -> Result<(), String> {
        let registered = self.registered_shortcuts.lock().unwrap();

        for shortcut_str in registered.iter() {
            if let Ok(shortcut) = shortcut_str.parse::<Shortcut>() {
                app.global_shortcut()
                    .unregister(shortcut)
                    .map_err(|e| format!("Failed to unregister {}: {}", shortcut_str, e))?;
            }
        }

        Ok(())
    }

    /// Update the hotkey configuration and re-register
    pub fn update_config(&self, app: &AppHandle, new_config: HotkeyConfig) -> Vec<Result<(), String>> {
        // Unregister existing hotkeys
        self.unregister_all(app).ok();

        // Update config
        *self.config.lock().unwrap() = new_config;

        // Re-register with new config
        self.register_all(app)
    }

    /// Get the current configuration
    pub fn get_config(&self) -> HotkeyConfig {
        self.config.lock().unwrap().clone()
    }

    /// Check if a shortcut is currently registered
    pub fn is_registered(&self, shortcut: &str) -> bool {
        self.registered_shortcuts
            .lock()
            .unwrap()
            .contains(&shortcut.to_string())
    }
}

impl Default for HotkeyManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hotkey_action_event_names() {
        assert_eq!(
            HotkeyAction::ToggleSession.event_name(),
            "hotkey-toggle-session"
        );
        assert_eq!(
            HotkeyAction::StartBugCapture.event_name(),
            "hotkey-start-bug-capture"
        );
        assert_eq!(
            HotkeyAction::EndBugCapture.event_name(),
            "hotkey-end-bug-capture"
        );
        assert_eq!(
            HotkeyAction::OpenQuickNotepad.event_name(),
            "hotkey-open-quick-notepad"
        );
        assert_eq!(
            HotkeyAction::OpenSessionNotepad.event_name(),
            "hotkey-open-session-notepad"
        );
    }

    #[test]
    fn test_hotkey_action_descriptions() {
        assert_eq!(HotkeyAction::ToggleSession.description(), "Toggle Session");
        assert_eq!(
            HotkeyAction::StartBugCapture.description(),
            "Start Bug Capture"
        );
        assert_eq!(HotkeyAction::EndBugCapture.description(), "End Bug Capture");
        assert_eq!(
            HotkeyAction::OpenQuickNotepad.description(),
            "Open Quick Notepad"
        );
        assert_eq!(
            HotkeyAction::OpenSessionNotepad.description(),
            "Open Session Notepad"
        );
    }

    #[test]
    fn test_default_config() {
        let config = HotkeyConfig::default();
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::ToggleSession),
            Some(&"Ctrl+Shift+Q".to_string())
        );
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::StartBugCapture),
            Some(&"PrintScreen".to_string())
        );
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::EndBugCapture),
            Some(&"F4".to_string())
        );
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::OpenQuickNotepad),
            Some(&"Ctrl+Shift+N".to_string())
        );
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::OpenSessionNotepad),
            Some(&"Ctrl+Shift+M".to_string())
        );
    }

    #[test]
    fn test_hotkey_manager_creation() {
        let manager = HotkeyManager::new();
        let config = manager.get_config();
        assert_eq!(config.shortcuts.len(), 5);
    }

    #[test]
    fn test_hotkey_manager_with_custom_config() {
        let mut shortcuts = HashMap::new();
        shortcuts.insert(HotkeyAction::ToggleSession, "Ctrl+Alt+T".to_string());
        let custom_config = HotkeyConfig { shortcuts };

        let manager = HotkeyManager::with_config(custom_config);
        let config = manager.get_config();
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::ToggleSession),
            Some(&"Ctrl+Alt+T".to_string())
        );
    }

    #[test]
    fn test_is_registered_initially_false() {
        let manager = HotkeyManager::new();
        assert!(!manager.is_registered("Ctrl+Shift+Q"));
    }

    #[test]
    fn test_config_serialization() {
        let config = HotkeyConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: HotkeyConfig = serde_json::from_str(&json).unwrap();

        assert_eq!(
            config.shortcuts.get(&HotkeyAction::ToggleSession),
            deserialized.shortcuts.get(&HotkeyAction::ToggleSession)
        );
    }

    #[test]
    fn test_hotkey_action_serialization() {
        let action = HotkeyAction::ToggleSession;
        let json = serde_json::to_string(&action).unwrap();
        assert_eq!(json, "\"toggle_session\"");

        let deserialized: HotkeyAction = serde_json::from_str(&json).unwrap();
        assert_eq!(action, deserialized);
    }

    #[test]
    fn test_all_actions_have_unique_event_names() {
        use std::collections::HashSet;
        let actions = [
            HotkeyAction::ToggleSession,
            HotkeyAction::StartBugCapture,
            HotkeyAction::EndBugCapture,
            HotkeyAction::OpenQuickNotepad,
            HotkeyAction::OpenSessionNotepad,
        ];

        let event_names: HashSet<_> = actions.iter().map(|a| a.event_name()).collect();
        assert_eq!(event_names.len(), 5);
    }

    #[test]
    fn test_all_actions_have_unique_descriptions() {
        use std::collections::HashSet;
        let actions = [
            HotkeyAction::ToggleSession,
            HotkeyAction::StartBugCapture,
            HotkeyAction::EndBugCapture,
            HotkeyAction::OpenQuickNotepad,
            HotkeyAction::OpenSessionNotepad,
        ];

        let descriptions: HashSet<_> = actions.iter().map(|a| a.description()).collect();
        assert_eq!(descriptions.len(), 5);
    }

    #[test]
    fn test_config_clone() {
        let config1 = HotkeyConfig::default();
        let config2 = config1.clone();

        assert_eq!(
            config1.shortcuts.get(&HotkeyAction::ToggleSession),
            config2.shortcuts.get(&HotkeyAction::ToggleSession)
        );
    }
}
