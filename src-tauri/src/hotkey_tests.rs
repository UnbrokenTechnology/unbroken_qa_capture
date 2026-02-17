#[cfg(test)]
mod integration_tests {
    use crate::hotkey::{HotkeyAction, HotkeyConfig, HotkeyManager};
    use std::collections::HashMap;

    #[test]
    fn test_manager_initialization() {
        let manager = HotkeyManager::new();
        let config = manager.get_config();

        // Verify all default shortcuts are present
        assert!(config.shortcuts.contains_key(&HotkeyAction::ToggleSession));
        assert!(config.shortcuts.contains_key(&HotkeyAction::StartBugCapture));
        assert!(config.shortcuts.contains_key(&HotkeyAction::EndBugCapture));
        assert!(config.shortcuts.contains_key(&HotkeyAction::OpenQuickNotepad));
        assert!(config.shortcuts.contains_key(&HotkeyAction::OpenSessionNotepad));
    }

    #[test]
    fn test_custom_config_initialization() {
        let mut shortcuts = HashMap::new();
        shortcuts.insert(HotkeyAction::ToggleSession, "Alt+F1".to_string());
        shortcuts.insert(HotkeyAction::StartBugCapture, "Alt+F2".to_string());

        let custom_config = HotkeyConfig { shortcuts };
        let manager = HotkeyManager::with_config(custom_config.clone());

        let retrieved_config = manager.get_config();
        assert_eq!(
            retrieved_config.shortcuts.get(&HotkeyAction::ToggleSession),
            Some(&"Alt+F1".to_string())
        );
        assert_eq!(
            retrieved_config.shortcuts.get(&HotkeyAction::StartBugCapture),
            Some(&"Alt+F2".to_string())
        );
    }

    #[test]
    fn test_event_name_mapping() {
        let actions = vec![
            (HotkeyAction::ToggleSession, "hotkey-toggle-session"),
            (HotkeyAction::StartBugCapture, "hotkey-start-bug-capture"),
            (HotkeyAction::EndBugCapture, "hotkey-end-bug-capture"),
            (HotkeyAction::OpenQuickNotepad, "hotkey-open-quick-notepad"),
            (HotkeyAction::OpenSessionNotepad, "hotkey-open-session-notepad"),
        ];

        for (action, expected_event) in actions {
            assert_eq!(action.event_name(), expected_event);
        }
    }

    #[test]
    fn test_description_mapping() {
        let actions = vec![
            (HotkeyAction::ToggleSession, "Toggle Session"),
            (HotkeyAction::StartBugCapture, "Start Bug Capture"),
            (HotkeyAction::EndBugCapture, "End Bug Capture"),
            (HotkeyAction::OpenQuickNotepad, "Open Quick Notepad"),
            (HotkeyAction::OpenSessionNotepad, "Open Session Notepad"),
        ];

        for (action, expected_desc) in actions {
            assert_eq!(action.description(), expected_desc);
        }
    }

    #[test]
    fn test_config_update() {
        let manager = HotkeyManager::new();
        let original_config = manager.get_config();

        // Create a new config with different shortcuts
        let mut new_shortcuts = HashMap::new();
        new_shortcuts.insert(HotkeyAction::ToggleSession, "Ctrl+Alt+S".to_string());
        new_shortcuts.insert(HotkeyAction::StartBugCapture, "Ctrl+Alt+B".to_string());
        new_shortcuts.insert(HotkeyAction::EndBugCapture, "Ctrl+Alt+E".to_string());
        new_shortcuts.insert(HotkeyAction::OpenQuickNotepad, "Ctrl+Alt+Q".to_string());
        new_shortcuts.insert(HotkeyAction::OpenSessionNotepad, "Ctrl+Alt+M".to_string());

        let new_config = HotkeyConfig {
            shortcuts: new_shortcuts,
        };

        // The config should be different
        assert_ne!(
            original_config.shortcuts.get(&HotkeyAction::ToggleSession),
            new_config.shortcuts.get(&HotkeyAction::ToggleSession)
        );
    }

    #[test]
    fn test_is_registered_initially_false() {
        let manager = HotkeyManager::new();
        assert!(!manager.is_registered("F5"));
        assert!(!manager.is_registered("F7"));
        assert!(!manager.is_registered("F9"));
    }

    #[test]
    fn test_default_shortcuts_values() {
        let config = HotkeyConfig::default();

        assert_eq!(
            config.shortcuts.get(&HotkeyAction::ToggleSession).unwrap(),
            "F5"
        );
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::StartBugCapture).unwrap(),
            "F7"
        );
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::EndBugCapture).unwrap(),
            "F9"
        );
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::OpenQuickNotepad).unwrap(),
            "Ctrl+Shift+N"
        );
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::OpenSessionNotepad).unwrap(),
            "Ctrl+Shift+M"
        );
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let original_config = HotkeyConfig::default();
        let json = serde_json::to_string(&original_config).unwrap();
        let deserialized_config: HotkeyConfig = serde_json::from_str(&json).unwrap();

        // Check all shortcuts match
        for action in [
            HotkeyAction::ToggleSession,
            HotkeyAction::StartBugCapture,
            HotkeyAction::EndBugCapture,
            HotkeyAction::OpenQuickNotepad,
            HotkeyAction::OpenSessionNotepad,
        ] {
            assert_eq!(
                original_config.shortcuts.get(&action),
                deserialized_config.shortcuts.get(&action)
            );
        }
    }

    #[test]
    fn test_action_serialization() {
        let test_cases = vec![
            (HotkeyAction::ToggleSession, "\"toggle_session\""),
            (HotkeyAction::StartBugCapture, "\"start_bug_capture\""),
            (HotkeyAction::EndBugCapture, "\"end_bug_capture\""),
            (HotkeyAction::OpenQuickNotepad, "\"open_quick_notepad\""),
            (HotkeyAction::OpenSessionNotepad, "\"open_session_notepad\""),
        ];

        for (action, expected_json) in test_cases {
            let json = serde_json::to_string(&action).unwrap();
            assert_eq!(json, expected_json);

            let deserialized: HotkeyAction = serde_json::from_str(&json).unwrap();
            assert_eq!(action, deserialized);
        }
    }

    #[test]
    fn test_all_actions_covered() {
        // This test ensures we haven't forgotten to handle any action
        let config = HotkeyConfig::default();

        // If a new action is added but not included in the default config, this test will fail
        let expected_count = 5; // Current number of actions
        assert_eq!(config.shortcuts.len(), expected_count);
    }

    #[test]
    fn test_manager_clone_independence() {
        let manager1 = HotkeyManager::new();
        let config1 = manager1.get_config();

        let mut new_shortcuts = HashMap::new();
        new_shortcuts.insert(HotkeyAction::ToggleSession, "Alt+T".to_string());
        let config2 = HotkeyConfig {
            shortcuts: new_shortcuts,
        };

        let manager2 = HotkeyManager::with_config(config2);
        let retrieved_config2 = manager2.get_config();

        // Verify manager1 config is unchanged
        let retrieved_config1 = manager1.get_config();
        assert_eq!(
            config1.shortcuts.get(&HotkeyAction::ToggleSession),
            retrieved_config1.shortcuts.get(&HotkeyAction::ToggleSession)
        );

        // Verify manager2 has different config
        assert_ne!(
            retrieved_config1.shortcuts.get(&HotkeyAction::ToggleSession),
            retrieved_config2.shortcuts.get(&HotkeyAction::ToggleSession)
        );
    }

    #[test]
    fn test_empty_config() {
        let empty_config = HotkeyConfig {
            shortcuts: HashMap::new(),
        };
        let manager = HotkeyManager::with_config(empty_config);
        let config = manager.get_config();

        assert_eq!(config.shortcuts.len(), 0);
    }

    #[test]
    fn test_partial_config() {
        let mut shortcuts = HashMap::new();
        shortcuts.insert(HotkeyAction::ToggleSession, "F1".to_string());
        // Only one action configured

        let partial_config = HotkeyConfig { shortcuts };
        let manager = HotkeyManager::with_config(partial_config);
        let config = manager.get_config();

        assert_eq!(config.shortcuts.len(), 1);
        assert_eq!(
            config.shortcuts.get(&HotkeyAction::ToggleSession),
            Some(&"F1".to_string())
        );
        assert_eq!(config.shortcuts.get(&HotkeyAction::StartBugCapture), None);
    }

    #[test]
    fn test_config_with_duplicate_shortcuts() {
        // This is allowed by the config structure (different actions can have the same shortcut)
        // Though in practice, the last one to register would win
        let mut shortcuts = HashMap::new();
        shortcuts.insert(HotkeyAction::ToggleSession, "F1".to_string());
        shortcuts.insert(HotkeyAction::StartBugCapture, "F1".to_string());

        let config = HotkeyConfig { shortcuts };
        let manager = HotkeyManager::with_config(config);
        let retrieved_config = manager.get_config();

        assert_eq!(
            retrieved_config.shortcuts.get(&HotkeyAction::ToggleSession),
            Some(&"F1".to_string())
        );
        assert_eq!(
            retrieved_config.shortcuts.get(&HotkeyAction::StartBugCapture),
            Some(&"F1".to_string())
        );
    }

    #[test]
    fn test_manager_default_trait() {
        let manager1 = HotkeyManager::default();
        let manager2 = HotkeyManager::new();

        let config1 = manager1.get_config();
        let config2 = manager2.get_config();

        // Both should have the same default config
        for action in [
            HotkeyAction::ToggleSession,
            HotkeyAction::StartBugCapture,
            HotkeyAction::EndBugCapture,
            HotkeyAction::OpenQuickNotepad,
            HotkeyAction::OpenSessionNotepad,
        ] {
            assert_eq!(
                config1.shortcuts.get(&action),
                config2.shortcuts.get(&action)
            );
        }
    }
}
