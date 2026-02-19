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

pub use types::{ClaudeError, ClaudeStatus, BugContext, PromptTask, ClaudeResponse, ClaudeRequest, ClaudeCredentials, TokenSource};
pub use subprocess::{ClaudeInvoker, RealClaudeInvoker};
pub use prompts::PromptBuilder;

/// Global Claude status
static CLAUDE_STATUS: Mutex<Option<ClaudeStatus>> = Mutex::new(None);

/// Load credentials: first check for an API key setting, then fall back to Claude Code OAuth token.
///
/// The `api_key_from_settings` parameter allows callers to pass in the API key
/// from the app's settings DB without this module needing direct DB access.
pub fn load_credentials(api_key_from_settings: Option<String>) -> Result<ClaudeCredentials, ClaudeError> {
    // 1. Check for an explicit Anthropic API key from settings
    if let Some(key) = api_key_from_settings {
        if !key.is_empty() {
            return Ok(ClaudeCredentials {
                access_token: key,
                token_source: TokenSource::ApiKey,
            });
        }
    }

    // 2. Fall back to Claude Code OAuth token (~/.claude/.credentials.json)
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
                                        token_source: TokenSource::OAuthToken,
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
                                token_source: TokenSource::OAuthToken,
                            });
                        }
                    }
                }
            }
        }
    }

    Err(ClaudeError::NotAuthenticated(
        "No API key configured and no Claude Code credentials found.".to_string()
    ))
}

/// Check if API credentials are available (without revealing the credentials themselves).
///
/// `api_key_from_settings`: the `anthropic_api_key` value from the settings DB, if any.
pub fn check_api_configured(api_key_from_settings: Option<String>) -> ClaudeStatus {
    match load_credentials(api_key_from_settings) {
        Ok(creds) => {
            let source_label = match creds.token_source {
                TokenSource::ApiKey => "API Key".to_string(),
                TokenSource::OAuthToken => "Claude Code OAuth".to_string(),
            };
            ClaudeStatus::Ready {
                version: source_label,
            }
        }
        Err(_) => ClaudeStatus::NotInstalled {
            message: "No API key configured. Enter your Anthropic API key in Settings, or install Claude Code for auto-detection.".to_string(),
        },
    }
}

/// Get cached Claude status or perform fresh check.
///
/// `api_key_from_settings`: the `anthropic_api_key` from the settings DB.
pub fn get_claude_status(api_key_from_settings: Option<String>) -> ClaudeStatus {
    // Try to use cached status first
    if let Some(status) = CLAUDE_STATUS.lock().unwrap().as_ref() {
        return status.clone();
    }

    // Perform fresh check
    let status = check_api_configured(api_key_from_settings);

    // Cache the result
    *CLAUDE_STATUS.lock().unwrap() = Some(status.clone());
    status
}

/// Refresh the cached Claude status
pub fn refresh_claude_status(api_key_from_settings: Option<String>) -> ClaudeStatus {
    *CLAUDE_STATUS.lock().unwrap() = None;
    get_claude_status(api_key_from_settings)
}

/// Invalidate the cached status so the next call re-checks credentials.
/// Called when the user saves or clears the API key.
pub fn invalidate_status_cache() {
    *CLAUDE_STATUS.lock().unwrap() = None;
}
