use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Session represents a QA testing session
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Session {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: SessionStatus,
    pub folder_path: String,
    pub session_notes: Option<String>,
    pub environment_json: Option<String>,
    pub original_snip_path: Option<String>,
    pub created_at: String,
}

/// Session status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Active,
    Ended,
    Reviewed,
    Synced,
}

impl SessionStatus {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            SessionStatus::Active => "active",
            SessionStatus::Ended => "ended",
            SessionStatus::Reviewed => "reviewed",
            SessionStatus::Synced => "synced",
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "active" => Ok(SessionStatus::Active),
            "ended" => Ok(SessionStatus::Ended),
            "reviewed" => Ok(SessionStatus::Reviewed),
            "synced" => Ok(SessionStatus::Synced),
            _ => Err(format!("Invalid session status: {}", s)),
        }
    }
}

/// Bug card represents an individual bug/issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bug {
    pub id: String,
    pub session_id: String,
    pub bug_number: i32,
    pub display_id: String,
    #[serde(rename = "type")]
    pub bug_type: BugType,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub description: Option<String>,
    pub ai_description: Option<String>,
    pub status: BugStatus,
    pub meeting_id: Option<String>,
    pub software_version: Option<String>,
    pub console_parse_json: Option<String>,
    pub metadata_json: Option<String>,
    /// Profile-driven custom field values stored as a JSON object (key → value).
    /// Replaces the fixed meeting_id / software_version fields for new bugs.
    /// Legacy fields are kept for backwards compatibility.
    pub custom_metadata: Option<String>,
    pub folder_path: String,
    pub created_at: String,
    pub updated_at: String,
}

/// Bug type enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BugType {
    Bug,
    Feature,
    Feedback,
}

impl BugType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            BugType::Bug => "bug",
            BugType::Feature => "feature",
            BugType::Feedback => "feedback",
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "bug" => Ok(BugType::Bug),
            "feature" => Ok(BugType::Feature),
            "feedback" => Ok(BugType::Feedback),
            _ => Err(format!("Invalid bug type: {}", s)),
        }
    }
}

/// Bug status enum
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BugStatus {
    Capturing,
    Captured,
    Reviewed,
    Ready,
}

impl BugStatus {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            BugStatus::Capturing => "capturing",
            BugStatus::Captured => "captured",
            BugStatus::Reviewed => "reviewed",
            BugStatus::Ready => "ready",
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "capturing" => Ok(BugStatus::Capturing),
            "captured" => Ok(BugStatus::Captured),
            "reviewed" => Ok(BugStatus::Reviewed),
            "ready" => Ok(BugStatus::Ready),
            _ => Err(format!("Invalid bug status: {}", s)),
        }
    }
}

/// Capture represents a media file (screenshot, video, console output)
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Capture {
    pub id: String,
    /// None when capture is unsorted (no active bug at capture time)
    pub bug_id: Option<String>,
    pub session_id: String,
    pub file_name: String,
    pub file_path: String,
    pub file_type: CaptureType,
    pub annotated_path: Option<String>,
    pub file_size_bytes: Option<i64>,
    pub is_console_capture: bool,
    pub parsed_content: Option<String>,
    pub created_at: String,
}

/// Capture type enum
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum CaptureType {
    Screenshot,
    Video,
    Console,
}

impl CaptureType {
    #[allow(dead_code)]
    pub fn as_str(&self) -> &str {
        match self {
            CaptureType::Screenshot => "screenshot",
            CaptureType::Video => "video",
            CaptureType::Console => "console",
        }
    }

    #[allow(dead_code)]
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "screenshot" => Ok(CaptureType::Screenshot),
            "video" => Ok(CaptureType::Video),
            "console" => Ok(CaptureType::Console),
            _ => Err(format!("Invalid capture type: {}", s)),
        }
    }
}

/// Setting represents a key-value configuration pair
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Setting {
    pub key: String,
    pub value: String,
    pub updated_at: String,
}

/// Environment metadata
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Environment {
    pub os: String,
    pub display_resolution: String,
    pub dpi_scaling: String,
    pub ram: String,
    pub cpu: String,
    pub foreground_app: String,
}

/// Bug metadata structure
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BugMetadata {
    pub meeting_id: Option<String>,
    pub software_version: Option<String>,
    pub environment: Environment,
    pub console_captures: Vec<String>,
    pub custom_fields: HashMap<String, String>,
}

/// Session summary for listings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SessionSummary {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub status: SessionStatus,
    pub bug_count: i32,
}

/// Bug update struct for partial updates
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BugUpdate {
    pub bug_type: Option<BugType>,
    pub title: Option<String>,
    pub notes: Option<String>,
    pub description: Option<String>,
    pub ai_description: Option<String>,
    pub status: Option<BugStatus>,
    pub meeting_id: Option<String>,
    pub software_version: Option<String>,
    /// Profile-driven custom field values stored as a JSON object (key → value).
    pub custom_metadata: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_status_conversions() {
        assert_eq!(SessionStatus::Active.as_str(), "active");
        assert_eq!(SessionStatus::from_str("active").unwrap(), SessionStatus::Active);
        assert_eq!(SessionStatus::from_str("ended").unwrap(), SessionStatus::Ended);
        assert!(SessionStatus::from_str("invalid").is_err());
    }

    #[test]
    fn test_bug_type_conversions() {
        assert_eq!(BugType::Bug.as_str(), "bug");
        assert_eq!(BugType::from_str("bug").unwrap(), BugType::Bug);
        assert_eq!(BugType::from_str("feature").unwrap(), BugType::Feature);
        assert!(BugType::from_str("invalid").is_err());
    }

    #[test]
    fn test_bug_status_conversions() {
        assert_eq!(BugStatus::Capturing.as_str(), "capturing");
        assert_eq!(BugStatus::from_str("captured").unwrap(), BugStatus::Captured);
        assert!(BugStatus::from_str("invalid").is_err());
    }

    #[test]
    fn test_capture_type_conversions() {
        assert_eq!(CaptureType::Screenshot.as_str(), "screenshot");
        assert_eq!(CaptureType::from_str("video").unwrap(), CaptureType::Video);
        assert!(CaptureType::from_str("invalid").is_err());
    }

    #[test]
    fn test_session_serialization() {
        let session = Session {
            id: "test-id".to_string(),
            started_at: "2024-01-01T00:00:00Z".to_string(),
            ended_at: None,
            status: SessionStatus::Active,
            folder_path: "/test/path".to_string(),
            session_notes: None,
            environment_json: None,
            original_snip_path: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&session).unwrap();
        let deserialized: Session = serde_json::from_str(&json).unwrap();
        assert_eq!(session, deserialized);
    }

    #[test]
    fn test_bug_serialization() {
        let bug = Bug {
            id: "bug-1".to_string(),
            session_id: "session-1".to_string(),
            bug_number: 1,
            display_id: "Bug-01".to_string(),
            bug_type: BugType::Bug,
            title: Some("Test bug".to_string()),
            notes: None,
            description: None,
            ai_description: None,
            status: BugStatus::Captured,
            meeting_id: None,
            software_version: None,
            console_parse_json: None,
            metadata_json: None,
            custom_metadata: None,
            folder_path: "/test/bug".to_string(),
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&bug).unwrap();
        let deserialized: Bug = serde_json::from_str(&json).unwrap();
        assert_eq!(bug, deserialized);
    }
}
