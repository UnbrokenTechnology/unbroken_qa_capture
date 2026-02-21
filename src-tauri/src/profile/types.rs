use serde::{Deserialize, Serialize};

/// QA testing profile — captures all configuration needed for a testing engagement
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct QaProfile {
    pub id: String,
    pub name: String,
    pub linear_config: Option<LinearProfileConfig>,
    pub area_categories: Vec<AreaCategory>,
    pub custom_fields: Vec<CustomMetadataField>,
    pub title_conventions: Option<TitleConventions>,
    pub created_at: String,
    pub updated_at: String,
}

/// Linear project management integration config per-profile
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LinearProfileConfig {
    pub team_id: String,
    pub api_key: String,
    pub default_assignee_id: Option<String>,
    pub default_bug_label_ids: Vec<String>,
    pub default_feature_label_ids: Vec<String>,
    pub default_state_id: Option<String>,
    /// Linear template ID to use when filing bug-type issues
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bug_template_id: Option<String>,
    /// Linear template ID to use when filing feature-type issues
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_template_id: Option<String>,
}

/// Area category used to classify bugs by functional area
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AreaCategory {
    pub code: String,
    pub name: String,
    pub description: Option<String>,
}

/// Custom metadata field definition for a profile
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CustomMetadataField {
    pub key: String,
    pub label: String,
    pub field_type: CustomFieldType,
    pub default_value: Option<String>,
    pub required: bool,
    /// Allowed option values for `Select`-type fields.
    /// Empty or absent means the options are populated at runtime (e.g. from `area_categories`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<Vec<String>>,
}

/// Supported field types for custom metadata
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CustomFieldType {
    Text,
    Number,
    Select,
}

impl CustomFieldType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            CustomFieldType::Text => "text",
            CustomFieldType::Number => "number",
            CustomFieldType::Select => "select",
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "text" => Ok(CustomFieldType::Text),
            "number" => Ok(CustomFieldType::Number),
            "select" => Ok(CustomFieldType::Select),
            _ => Err(format!("Invalid custom field type: {}", s)),
        }
    }
}

/// Naming conventions for generated issue titles
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TitleConventions {
    pub bug_prefix: String,
    pub feature_prefix: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_field_type_conversions() {
        assert_eq!(CustomFieldType::Text.as_str(), "text");
        assert_eq!(CustomFieldType::Number.as_str(), "number");
        assert_eq!(CustomFieldType::Select.as_str(), "select");

        assert_eq!(CustomFieldType::from_str("text").unwrap(), CustomFieldType::Text);
        assert_eq!(CustomFieldType::from_str("number").unwrap(), CustomFieldType::Number);
        assert_eq!(CustomFieldType::from_str("select").unwrap(), CustomFieldType::Select);
        assert!(CustomFieldType::from_str("invalid").is_err());
    }

    #[test]
    fn test_qa_profile_serialization() {
        let profile = QaProfile {
            id: "profile-1".to_string(),
            name: "Test Profile".to_string(),
            linear_config: None,
            area_categories: vec![
                AreaCategory {
                    code: "UI".to_string(),
                    name: "User Interface".to_string(),
                    description: Some("UI-related bugs".to_string()),
                },
            ],
            custom_fields: vec![
                CustomMetadataField {
                    key: "severity".to_string(),
                    label: "Severity".to_string(),
                    field_type: CustomFieldType::Select,
                    default_value: Some("medium".to_string()),
                    required: false,
                    options: Some(vec!["low".to_string(), "medium".to_string(), "high".to_string()]),
                },
            ],
            title_conventions: Some(TitleConventions {
                bug_prefix: "[BUG]".to_string(),
                feature_prefix: "[FEAT]".to_string(),
            }),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&profile).unwrap();
        let deserialized: QaProfile = serde_json::from_str(&json).unwrap();
        assert_eq!(profile, deserialized);
    }

    #[test]
    fn test_custom_field_type_serde_lowercase() {
        let field_type = CustomFieldType::Select;
        let json = serde_json::to_string(&field_type).unwrap();
        assert_eq!(json, "\"select\"");

        let deserialized: CustomFieldType = serde_json::from_str("\"text\"").unwrap();
        assert_eq!(deserialized, CustomFieldType::Text);
    }
}
