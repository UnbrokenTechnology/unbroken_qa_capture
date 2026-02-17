//! Claude CLI Integration Module
//!
//! This module provides subprocess-based integration with the Claude Code CLI
//! for AI-powered bug description generation and console screenshot parsing.
//!
//! Key features:
//! - Check CLI availability (claude --version)
//! - Check CLI authentication status
//! - Construct focused prompts with bug data (screenshots, notes, metadata)
//! - Invoke 'claude --print' with timeout and error handling
//! - Queue multiple requests (max 1 concurrent subprocess)
//! - Parse and return responses
//! - Graceful degradation when CLI not available

use std::process::{Command, Stdio};
use std::sync::Mutex;
use std::path::PathBuf;

mod types;
mod subprocess;
mod prompts;

#[cfg(test)]
mod tests;

pub use types::{ClaudeError, ClaudeStatus, BugContext, PromptTask, ClaudeResponse, ClaudeRequest};
pub use subprocess::{ClaudeInvoker, RealClaudeInvoker};
pub use prompts::PromptBuilder;

/// Global Claude CLI status
static CLAUDE_STATUS: Mutex<Option<ClaudeStatus>> = Mutex::new(None);

/// Find the Claude CLI executable
/// Tries PATH first, then falls back to common installation locations on Windows
pub(crate) fn find_claude_executable() -> Option<PathBuf> {
    // Try PATH first (works when running from terminal)
    // Remove CLAUDECODE env var to avoid "nested session" detection
    if let Ok(output) = Command::new("claude")
        .arg("--version")
        .env_remove("CLAUDECODE")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .and_then(|mut child| child.wait())
    {
        if output.success() {
            return Some(PathBuf::from("claude"));
        }
    }

    // On Windows, check explicit fallback locations
    // The GUI app may not inherit PATH that includes ~/.local/bin
    #[cfg(target_os = "windows")]
    {
        if let Some(home_dir) = dirs::home_dir() {
            let candidates = vec![
                home_dir.join(".local").join("bin").join("claude.exe"),
                home_dir.join(".claude").join("local").join("claude.exe"),
            ];

            for path in candidates {
                if path.exists() {
                    return Some(path);
                }
            }
        }
    }

    None
}

/// Check if Claude CLI is installed and available on PATH
pub fn check_cli_available() -> Result<String, ClaudeError> {
    let claude_path = find_claude_executable()
        .ok_or_else(|| ClaudeError::NotFound(
            "Claude CLI not found. Install from https://claude.ai/download".to_string()
        ))?;

    let output = Command::new(&claude_path)
        .arg("--version")
        .env_remove("CLAUDECODE")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| ClaudeError::NotFound(format!("Failed to spawn claude process: {}", e)))?
        .wait_with_output()
        .map_err(|e| ClaudeError::NotFound(format!("Failed to wait for claude process: {}", e)))?;

    if !output.status.success() {
        return Err(ClaudeError::NotFound(
            "claude command found but failed to execute".to_string()
        ));
    }

    let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(version)
}

/// Check if Claude CLI is authenticated by looking for credential files.
/// Avoids invoking `claude --print` which would cost API credits just for a status check.
/// Returns Ok(()) if credentials found, Err if not.
pub fn check_cli_authenticated() -> Result<(), ClaudeError> {
    if let Some(home_dir) = dirs::home_dir() {
        let credentials_path = home_dir.join(".claude").join(".credentials.json");
        if credentials_path.exists() {
            // Check file is non-empty (not just an empty placeholder)
            if let Ok(metadata) = std::fs::metadata(&credentials_path) {
                if metadata.len() > 2 {
                    return Ok(());
                }
            }
        }
    }

    Err(ClaudeError::NotAuthenticated(
        "No credentials found. Run 'claude' in a terminal to authenticate.".to_string()
    ))
}

/// Get cached Claude CLI status or perform fresh check
pub fn get_claude_status() -> ClaudeStatus {
    // Try to use cached status first
    if let Some(status) = CLAUDE_STATUS.lock().unwrap().as_ref() {
        return status.clone();
    }

    // Perform fresh check
    let status = match check_cli_available() {
        Ok(version) => {
            match check_cli_authenticated() {
                Ok(_) => ClaudeStatus::Ready { version },
                Err(e) => ClaudeStatus::NotAuthenticated {
                    version,
                    message: e.to_string()
                },
            }
        }
        Err(e) => ClaudeStatus::NotInstalled {
            message: e.to_string()
        },
    };

    // Cache the result
    *CLAUDE_STATUS.lock().unwrap() = Some(status.clone());
    status
}

/// Refresh the cached Claude CLI status
pub fn refresh_claude_status() -> ClaudeStatus {
    *CLAUDE_STATUS.lock().unwrap() = None;
    get_claude_status()
}

