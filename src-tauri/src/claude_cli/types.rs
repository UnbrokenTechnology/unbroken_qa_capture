//! Type definitions for Claude CLI / Anthropic API integration

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Credentials for calling the Anthropic Messages API via Claude Code OAuth
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClaudeCredentials {
    pub access_token: String,
}

/// Claude CLI availability status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum ClaudeStatus {
    /// CLI is installed and authenticated, ready to use
    Ready {
        version: String,
    },
    /// CLI is installed but not authenticated
    NotAuthenticated {
        version: String,
        message: String,
    },
    /// CLI is not installed
    NotInstalled {
        message: String,
    },
}

impl ClaudeStatus {
    #[allow(dead_code)]
    pub fn is_ready(&self) -> bool {
        matches!(self, ClaudeStatus::Ready { .. })
    }
}

/// Errors that can occur during Claude CLI operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "camelCase")]
pub enum ClaudeError {
    /// CLI executable not found on PATH
    NotFound(String),
    /// CLI found but not authenticated
    NotAuthenticated(String),
    /// Subprocess invocation failed
    InvocationFailed(String),
    /// Subprocess timed out
    Timeout {
        seconds: u64,
        task: String,
    },
    /// Failed to parse CLI output
    ParseError(String),
    /// Rate limit or API error from Claude
    ApiError(String),
    /// Queue is full
    QueueFull(String),
}

impl std::fmt::Display for ClaudeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaudeError::NotFound(msg) => write!(f, "Claude CLI not found: {}", msg),
            ClaudeError::NotAuthenticated(msg) => write!(f, "Claude CLI not authenticated: {}", msg),
            ClaudeError::InvocationFailed(msg) => write!(f, "Claude invocation failed: {}", msg),
            ClaudeError::Timeout { seconds, task } => {
                write!(f, "Claude invocation timed out after {}s for task: {}", seconds, task)
            }
            ClaudeError::ParseError(msg) => write!(f, "Failed to parse Claude response: {}", msg),
            ClaudeError::ApiError(msg) => write!(f, "Claude API error: {}", msg),
            ClaudeError::QueueFull(msg) => write!(f, "Claude request queue full: {}", msg),
        }
    }
}

impl std::error::Error for ClaudeError {}

/// Context data for a bug to be described by Claude
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BugContext {
    /// Bug ID
    pub bug_id: String,
    /// Quick notes from capture
    pub notes: Option<String>,
    /// Paths to screenshot files (max 5 recommended)
    pub screenshot_paths: Vec<PathBuf>,
    /// Application name
    pub app_name: Option<String>,
    /// Application version
    pub app_version: Option<String>,
    /// Meeting/session ID
    pub meeting_id: Option<String>,
    /// Environment info
    pub environment: Option<String>,
    /// Bug type (bug, feature, feedback)
    pub bug_type: Option<String>,
}

/// The type of AI task to perform
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PromptTask {
    /// Generate initial bug description from screenshots and notes
    DescribeBug,
    /// Parse console screenshot for errors/warnings
    ParseConsole,
    /// Refine existing description based on user instructions
    RefineDescription,
    /// Custom task with user-provided prompt
    Custom,
}

/// AI suggestion for which bug a capture belongs to
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CaptureAssignmentSuggestion {
    pub capture_id: String,
    /// None means "this looks like a new bug"
    pub suggested_bug_id: Option<String>,
    /// For UI display (e.g. "BUG-001")
    pub suggested_bug_display_id: Option<String>,
    /// 0.0–1.0 confidence score
    pub confidence: f32,
    pub reasoning: String,
}

/// Response from Claude CLI invocation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeResponse {
    /// The generated content from Claude
    pub content: String,
    /// Task type that was performed
    pub task: PromptTask,
    /// Bug ID this response is for (if applicable)
    pub bug_id: Option<String>,
}

/// Request to invoke Claude CLI
#[derive(Debug, Clone)]
pub struct ClaudeRequest {
    /// The prompt text to send
    pub prompt: String,
    /// Image file paths to attach (optional)
    pub image_paths: Vec<PathBuf>,
    /// Task type
    pub task: PromptTask,
    /// Bug ID (for tracking)
    pub bug_id: Option<String>,
    /// Timeout in seconds (15 for text, 30 for images)
    pub timeout_secs: u64,
}

impl ClaudeRequest {
    pub fn new_text(prompt: String, task: PromptTask) -> Self {
        Self {
            prompt,
            image_paths: Vec::new(),
            task,
            bug_id: None,
            timeout_secs: 15,
        }
    }

    pub fn new_with_images(
        prompt: String,
        image_paths: Vec<PathBuf>,
        task: PromptTask,
    ) -> Self {
        Self {
            prompt,
            image_paths,
            task,
            bug_id: None,
            timeout_secs: 30,
        }
    }

    pub fn with_bug_id(mut self, bug_id: String) -> Self {
        self.bug_id = Some(bug_id);
        self
    }

    #[allow(dead_code)]
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }
}
