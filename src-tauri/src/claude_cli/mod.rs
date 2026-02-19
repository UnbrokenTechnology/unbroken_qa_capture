//! Claude API Integration Module
//!
//! This module provides integration with the Anthropic Messages API
//! for AI-powered bug description generation and console screenshot parsing.
//!
//! Key features:
//! - Check API credential availability (API key or Claude Code OAuth)
//! - Construct focused prompts with bug data (screenshots, notes, metadata)
//! - Call Anthropic Messages API with timeout and error handling
//! - Queue multiple requests (max 1 concurrent)
//! - Parse and return responses
//! - Graceful degradation when no credentials configured

use std::sync::Mutex;

mod types;
mod subprocess;
mod prompts;

#[cfg(test)]
mod tests;

pub use types::{ClaudeError, ClaudeStatus, BugContext, PromptTask, ClaudeResponse, ClaudeRequest, ClaudeCredentials};
pub use subprocess::{ClaudeInvoker, RealClaudeInvoker};
pub use prompts::PromptBuilder;

/// Global Claude status
static CLAUDE_STATUS: Mutex<Option<ClaudeStatus>> = Mutex::new(None);

/// Load credentials from Claude Code OAuth token (~/.claude/.credentials.json).
///
/// Uses the Claude subscription (via Claude Code) — no API key needed.
pub fn load_credentials() -> Result<ClaudeCredentials, ClaudeError> {
    if let Some(home_dir) = dirs::home_dir() {
        let credentials_path = home_dir.join(".claude").join(".credentials.json");
        if credentials_path.exists() {
            if let Ok(contents) = std::fs::read_to_string(&credentials_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&contents) {
                    // The credentials file stores { "claudeAiOauth": { "<url>": { "accessToken": "...", ... } } }
                    // or it may have a top-level "accessToken" field.
                    // Try nested structure first (Claude Code format)
                    if let Some(oauth_obj) = json.get("claudeAiOauth").and_then(|v| v.as_object()) {
                        for (_key, entry) in oauth_obj {
                            if let Some(token) = entry.get("accessToken").and_then(|v| v.as_str()) {
                                if !token.is_empty() {
                                    return Ok(ClaudeCredentials {
                                        access_token: token.to_string(),
                                    });
                                }
                            }
                        }
                    }
                    // Try flat structure
                    if let Some(token) = json.get("accessToken").and_then(|v| v.as_str()) {
                        if !token.is_empty() {
                            return Ok(ClaudeCredentials {
                                access_token: token.to_string(),
                            });
                        }
                    }
                }
            }
        }
    }

    Err(ClaudeError::NotAuthenticated(
        "Claude Code not found. Install Claude Code and sign in to use AI features.".to_string()
    ))
}

/// Check if Claude Code OAuth credentials are available.
pub fn check_api_configured() -> ClaudeStatus {
    match load_credentials() {
        Ok(_) => ClaudeStatus::Ready {
            version: "Claude Code".to_string(),
        },
        Err(_) => ClaudeStatus::NotInstalled {
            message: "Claude Code not found. Install Claude Code and sign in to use AI features with your Claude subscription.".to_string(),
        },
    }
}

/// Get cached Claude status or perform fresh check.
pub fn get_claude_status() -> ClaudeStatus {
    // Try to use cached status first
    if let Some(status) = CLAUDE_STATUS.lock().unwrap().as_ref() {
        return status.clone();
    }

    // Perform fresh check
    let status = check_api_configured();

    // Cache the result
    *CLAUDE_STATUS.lock().unwrap() = Some(status.clone());
    status
}

/// Refresh the cached Claude status
pub fn refresh_claude_status() -> ClaudeStatus {
    *CLAUDE_STATUS.lock().unwrap() = None;
    get_claude_status()
}
