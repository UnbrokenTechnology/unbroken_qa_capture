use notify::{Watcher, RecursiveMode, Event, EventKind};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub const DEFAULT_TEMPLATE: &str = include_str!("../templates/default_template.md");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugMetadata {
    pub meeting_id: Option<String>,
    pub software_version: Option<String>,
    pub environment: Environment,
    pub console_captures: Vec<String>,
    pub custom_fields: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Environment {
    pub os: String,
    pub display_resolution: String,
    pub dpi_scaling: String,
    pub ram: String,
    pub cpu: String,
    pub foreground_app: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BugData {
    pub title: String,
    pub bug_type: String,
    pub description_steps: String,
    pub description_expected: String,
    pub description_actual: String,
    pub metadata: BugMetadata,
    pub folder_path: String,
    pub captures: Vec<String>,
    pub console_output: Option<String>,
}

/// Template manager handles loading, caching, and hot-reloading of ticket templates
pub struct TemplateManager {
    pub custom_template_path: Option<PathBuf>,
    cached_template: Arc<Mutex<String>>,
    watcher: Option<notify::RecommendedWatcher>,
}

impl TemplateManager {
    pub fn new() -> Self {
        Self {
            custom_template_path: None,
            cached_template: Arc::new(Mutex::new(DEFAULT_TEMPLATE.to_string())),
            watcher: None,
        }
    }

    /// Set the path to a custom template file
    pub fn set_custom_template_path(&mut self, path: Option<PathBuf>) -> Result<(), String> {
        self.custom_template_path = path.clone();

        if let Some(path) = path {
            // Load the custom template
            self.reload_template()?;

            // Start watching the file
            self.start_watching(&path)?;
        } else {
            // Reset to default template
            self.stop_watching();
            *self.cached_template.lock().unwrap() = DEFAULT_TEMPLATE.to_string();
        }

        Ok(())
    }

    /// Reload the template from disk
    pub fn reload_template(&self) -> Result<(), String> {
        let template_content = if let Some(path) = &self.custom_template_path {
            std::fs::read_to_string(path)
                .map_err(|e| format!("Failed to read custom template: {}", e))?
        } else {
            DEFAULT_TEMPLATE.to_string()
        };

        *self.cached_template.lock().unwrap() = template_content;
        Ok(())
    }

    /// Start watching the template file for changes
    fn start_watching(&mut self, path: &Path) -> Result<(), String> {
        let cached_template = Arc::clone(&self.cached_template);
        let path_clone = path.to_path_buf();

        let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        if let Ok(content) = std::fs::read_to_string(&path_clone) {
                            *cached_template.lock().unwrap() = content;
                            println!("Template reloaded: {:?}", path_clone);
                        }
                    }
                }
                Err(e) => eprintln!("Watch error: {:?}", e),
            }
        })
        .map_err(|e| format!("Failed to create watcher: {}", e))?;

        watcher.watch(path, RecursiveMode::NonRecursive)
            .map_err(|e| format!("Failed to watch template file: {}", e))?;

        self.watcher = Some(watcher);
        Ok(())
    }

    /// Stop watching the template file
    fn stop_watching(&mut self) {
        self.watcher = None;
    }

    /// Render a bug using the current template
    pub fn render(&self, bug: &BugData) -> Result<String, String> {
        let template = self.cached_template.lock().unwrap().clone();
        let mut output = template;

        // Simple placeholder replacement
        output = output.replace("{bug.title}", &bug.title);
        output = output.replace("{bug.type}", &bug.bug_type);
        output = output.replace("{bug.description.steps}", &bug.description_steps);
        output = output.replace("{bug.description.expected}", &bug.description_expected);
        output = output.replace("{bug.description.actual}", &bug.description_actual);
        output = output.replace("{bug.folderPath}", &bug.folder_path);

        // Metadata fields
        output = output.replace("{bug.metadata.environment.os}", &bug.metadata.environment.os);
        output = output.replace("{bug.metadata.environment.displayResolution}", &bug.metadata.environment.display_resolution);
        output = output.replace("{bug.metadata.environment.dpiScaling}", &bug.metadata.environment.dpi_scaling);
        output = output.replace("{bug.metadata.environment.foregroundApp}", &bug.metadata.environment.foreground_app);

        let version = bug.metadata.software_version.as_deref().unwrap_or("Unknown");
        output = output.replace("{bug.metadata.softwareVersion}", version);

        // Conditional fields (meeting ID)
        output = Self::replace_conditional(&output, "bug.metadata.meetingId", &bug.metadata.meeting_id);

        // Captures
        let captures_count = bug.captures.len().to_string();
        output = output.replace("{bug.captures.count}", &captures_count);

        let captures_list = bug.captures.iter()
            .map(|c| format!("- {}", c))
            .collect::<Vec<_>>()
            .join("\n");
        output = output.replace("{bug.captures.list}", &captures_list);

        // Console output
        let console_output = bug.console_output.as_deref().unwrap_or("No console output captured");
        output = output.replace("{bug.consoleOutput}", console_output);

        Ok(output)
    }

    /// Replace conditional placeholders (lines that should only appear if value exists)
    fn replace_conditional(template: &str, field: &str, value: &Option<String>) -> String {
        let pattern = format!("{{{}:", field);
        let mut result = String::new();

        for line in template.lines() {
            if line.contains(&pattern) {
                if let Some(val) = value {
                    // Find the conditional pattern and extract the template inside
                    // Format: {field:- **Label:** {value}}
                    // We need to find the matching closing brace for the outer pattern
                    if let Some(start_pos) = line.find(&pattern) {
                        let prefix = &line[..start_pos];
                        let rest = &line[start_pos + pattern.len()..];

                        // Find the matching closing brace (need to handle nested braces)
                        let mut depth = 0;
                        let mut end_pos = 0;
                        for (i, ch) in rest.chars().enumerate() {
                            match ch {
                                '{' => depth += 1,
                                '}' => {
                                    if depth == 0 {
                                        end_pos = i;
                                        break;
                                    }
                                    depth -= 1;
                                }
                                _ => {}
                            }
                        }

                        let inner_template = &rest[..end_pos];
                        let processed_line = format!("{}{}", prefix, inner_template.replace("{value}", val));
                        result.push_str(&processed_line);
                        result.push('\n');
                    }
                }
                // If no value, skip the entire line
            } else {
                result.push_str(line);
                result.push('\n');
            }
        }

        result
    }
}

impl Drop for TemplateManager {
    fn drop(&mut self) {
        self.stop_watching();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_bug() -> BugData {
        BugData {
            title: "Test Bug".to_string(),
            bug_type: "UI".to_string(),
            description_steps: "1. Click button\n2. Observe error".to_string(),
            description_expected: "Button should work".to_string(),
            description_actual: "Button crashes app".to_string(),
            metadata: BugMetadata {
                meeting_id: Some("MTG-123".to_string()),
                software_version: Some("1.0.0".to_string()),
                environment: Environment {
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
            folder_path: "/path/to/bug".to_string(),
            captures: vec!["screenshot1.png".to_string(), "screenshot2.png".to_string()],
            console_output: Some("Error: Something went wrong".to_string()),
        }
    }

    #[test]
    fn test_template_manager_new() {
        let manager = TemplateManager::new();
        assert!(manager.custom_template_path.is_none());
    }

    #[test]
    fn test_render_with_default_template() {
        let manager = TemplateManager::new();
        let bug = create_test_bug();
        let result = manager.render(&bug);

        assert!(result.is_ok());
        let output = result.unwrap();

        println!("Rendered output:\n{}", output);

        assert!(output.contains("Test Bug"));
        assert!(output.contains("UI"));
        assert!(output.contains("Click button"));
        assert!(output.contains("Windows 11"));
        assert!(output.contains("MTG-123"));
        assert!(output.contains("2 file(s)"));
    }

    #[test]
    fn test_conditional_field_with_value() {
        let bug = create_test_bug();
        let manager = TemplateManager::new();
        let result = manager.render(&bug).unwrap();

        // Meeting ID should appear
        assert!(result.contains("MTG-123"));
    }

    #[test]
    fn test_conditional_field_without_value() {
        let mut bug = create_test_bug();
        bug.metadata.meeting_id = None;

        let manager = TemplateManager::new();
        let result = manager.render(&bug).unwrap();

        // Line with meeting ID should not appear
        assert!(!result.contains("Meeting ID:"));
    }

    #[test]
    fn test_captures_list() {
        let bug = create_test_bug();
        let manager = TemplateManager::new();
        let result = manager.render(&bug).unwrap();

        assert!(result.contains("screenshot1.png"));
        assert!(result.contains("screenshot2.png"));
        assert!(result.contains("2 file(s)"));
    }
}
