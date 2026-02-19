use serde::{Deserialize, Serialize};

/// Result type for ticketing operations
pub type TicketingResult<T> = Result<T, TicketingError>;

/// Errors that can occur during ticketing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TicketingError {
    /// Authentication failed
    AuthenticationFailed(String),
    /// Network or API error
    NetworkError(String),
    /// Invalid configuration
    InvalidConfig(String),
    /// Ticket creation failed
    CreationFailed(String),
    /// Connection check failed
    ConnectionFailed(String),
}

impl std::fmt::Display for TicketingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AuthenticationFailed(msg) => write!(f, "Authentication failed: {}", msg),
            Self::NetworkError(msg) => write!(f, "Network error: {}", msg),
            Self::InvalidConfig(msg) => write!(f, "Invalid configuration: {}", msg),
            Self::CreationFailed(msg) => write!(f, "Ticket creation failed: {}", msg),
            Self::ConnectionFailed(msg) => write!(f, "Connection check failed: {}", msg),
        }
    }
}

impl std::error::Error for TicketingError {}

/// Credentials for a ticketing integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketingCredentials {
    /// API key or token
    pub api_key: String,
    /// Optional workspace/org ID
    pub workspace_id: Option<String>,
    /// Optional team ID (for Linear)
    pub team_id: Option<String>,
}

/// Request to create a ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTicketRequest {
    /// Ticket title
    pub title: String,
    /// Ticket description/body
    pub description: String,
    /// File paths to attach (screenshots, logs, etc.)
    pub attachments: Vec<String>,
    /// Optional priority level
    pub priority: Option<String>,
    /// Optional labels/tags
    pub labels: Vec<String>,
    /// Optional Linear assignee ID (from profile defaults)
    pub assignee_id: Option<String>,
    /// Optional Linear workflow state ID (from profile defaults)
    pub state_id: Option<String>,
}

/// Result of uploading a single attachment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttachmentUploadResult {
    /// Original file path that was uploaded
    pub file_path: String,
    /// Whether the upload succeeded
    pub success: bool,
    /// Asset URL on success, error message on failure
    pub message: String,
}

/// Response from creating a ticket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateTicketResponse {
    /// ID of the created ticket
    pub id: String,
    /// URL to view the ticket
    pub url: String,
    /// Display identifier (e.g., "PROJ-123")
    pub identifier: String,
    /// Results of attachment uploads (one entry per attachment in the request)
    pub attachment_results: Vec<AttachmentUploadResult>,
}

/// Connection status for a ticketing integration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    /// Whether connection is working
    pub connected: bool,
    /// Optional message (error details if not connected)
    pub message: Option<String>,
    /// Name of the integration
    pub integration_name: String,
}
