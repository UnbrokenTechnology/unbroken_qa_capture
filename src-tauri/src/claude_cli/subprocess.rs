//! Subprocess management for Claude CLI invocation
//!
//! Handles:
//! - Queueing requests (max 1 concurrent)
//! - Timeout enforcement
//! - Process cleanup
//! - Stdout/stderr capture

use super::types::{ClaudeError, ClaudeRequest, ClaudeResponse};
use std::collections::VecDeque;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Trait for invoking Claude CLI (enables mocking in tests)
pub trait ClaudeInvoker: Send + Sync {
    fn invoke(&self, request: ClaudeRequest) -> Result<ClaudeResponse, ClaudeError>;
}

/// Real implementation that spawns actual Claude CLI subprocess
pub struct RealClaudeInvoker;

impl RealClaudeInvoker {
    pub fn new() -> Self {
        Self
    }

    /// Spawn claude subprocess with the given request
    fn spawn_claude(&self, request: &ClaudeRequest) -> Result<Child, ClaudeError> {
        // Find the Claude CLI executable (supports Windows fallback locations)
        let claude_path = super::find_claude_executable()
            .ok_or_else(|| ClaudeError::NotFound(
                "Claude CLI not found. Install from https://claude.ai/download".to_string()
            ))?;

        let mut cmd = Command::new(&claude_path);
        cmd.args(["--print", "--output-format", "json"]);

        // Add image files if present
        for img_path in &request.image_paths {
            cmd.arg("--file");
            cmd.arg(img_path);
        }

        // Add the prompt text as a positional argument
        cmd.arg(&request.prompt);

        // Setup I/O
        cmd.stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        cmd.spawn()
            .map_err(|e| ClaudeError::InvocationFailed(format!("Failed to spawn claude: {}", e)))
    }

    /// Wait for process with timeout
    fn wait_with_timeout(
        &self,
        mut child: Child,
        timeout_secs: u64,
    ) -> Result<(String, String), ClaudeError> {
        let start = Instant::now();
        let timeout = Duration::from_secs(timeout_secs);

        loop {
            match child.try_wait() {
                Ok(Some(status)) => {
                    // Process finished
                    let mut stdout = String::new();
                    let mut stderr = String::new();

                    if let Some(mut out) = child.stdout.take() {
                        use std::io::Read;
                        out.read_to_string(&mut stdout)
                            .map_err(|e| ClaudeError::InvocationFailed(format!("Failed to read stdout: {}", e)))?;
                    }

                    if let Some(mut err) = child.stderr.take() {
                        use std::io::Read;
                        err.read_to_string(&mut stderr)
                            .map_err(|e| ClaudeError::InvocationFailed(format!("Failed to read stderr: {}", e)))?;
                    }

                    if !status.success() {
                        return Err(ClaudeError::InvocationFailed(format!(
                            "Claude exited with status {}: {}",
                            status, stderr
                        )));
                    }

                    return Ok((stdout, stderr));
                }
                Ok(None) => {
                    // Still running, check timeout
                    if start.elapsed() >= timeout {
                        // Timeout - kill the process
                        let _ = child.kill();
                        let _ = child.wait(); // Clean up zombie

                        return Err(ClaudeError::Timeout {
                            seconds: timeout_secs,
                            task: format!("{:?}", child),
                        });
                    }

                    // Sleep briefly before checking again
                    thread::sleep(Duration::from_millis(100));
                }
                Err(e) => {
                    return Err(ClaudeError::InvocationFailed(format!(
                        "Error waiting for process: {}",
                        e
                    )));
                }
            }
        }
    }
}

impl ClaudeInvoker for RealClaudeInvoker {
    fn invoke(&self, request: ClaudeRequest) -> Result<ClaudeResponse, ClaudeError> {
        // Spawn the process
        let child = self.spawn_claude(&request)?;

        // Wait with timeout
        let (stdout, _stderr) = self.wait_with_timeout(child, request.timeout_secs)?;

        // Parse response
        // For --output-format json, Claude returns JSON with a "content" field
        // But we'll also handle plain text responses
        let content = if stdout.trim().starts_with('{') {
            // Try to parse as JSON
            match serde_json::from_str::<serde_json::Value>(&stdout) {
                Ok(json) => {
                    // Extract content field if present
                    json.get("content")
                        .and_then(|v| v.as_str())
                        .unwrap_or(stdout.trim())
                        .to_string()
                }
                Err(_) => {
                    // Not valid JSON despite starting with {, use as-is
                    stdout.trim().to_string()
                }
            }
        } else {
            // Plain text response
            stdout.trim().to_string()
        };

        Ok(ClaudeResponse {
            content,
            task: request.task,
            bug_id: request.bug_id,
        })
    }
}

/// Queued invoker that ensures max 1 concurrent subprocess
/// Note: This is a placeholder for future async implementation
#[allow(dead_code)]
pub struct QueuedClaudeInvoker {
    inner: Arc<dyn ClaudeInvoker>,
    queue: Arc<Mutex<VecDeque<ClaudeRequest>>>,
    running: Arc<Mutex<bool>>,
    max_queue_size: usize,
}

#[allow(dead_code)]
impl QueuedClaudeInvoker {
    pub fn new(inner: Arc<dyn ClaudeInvoker>) -> Self {
        Self {
            inner,
            queue: Arc::new(Mutex::new(VecDeque::new())),
            running: Arc::new(Mutex::new(false)),
            max_queue_size: 10, // Reasonable limit
        }
    }

    pub fn with_max_queue_size(mut self, size: usize) -> Self {
        self.max_queue_size = size;
        self
    }

    /// Process the queue on a background thread
    fn process_queue(&self) {
        let inner = Arc::clone(&self.inner);
        let queue = Arc::clone(&self.queue);
        let running = Arc::clone(&self.running);

        thread::spawn(move || {
            loop {
                // Get next request from queue
                let request = {
                    let mut q = queue.lock().unwrap();
                    q.pop_front()
                };

                match request {
                    Some(req) => {
                        // Process it (blocking)
                        let _result = inner.invoke(req);
                        // Note: In a real implementation, we'd send result back via channel
                    }
                    None => {
                        // Queue empty, mark as not running and exit
                        *running.lock().unwrap() = false;
                        break;
                    }
                }
            }
        });
    }
}

impl ClaudeInvoker for QueuedClaudeInvoker {
    fn invoke(&self, request: ClaudeRequest) -> Result<ClaudeResponse, ClaudeError> {
        // Check if queue is full
        let queue_len = self.queue.lock().unwrap().len();
        if queue_len >= self.max_queue_size {
            return Err(ClaudeError::QueueFull(format!(
                "Request queue is full ({} items)",
                queue_len
            )));
        }

        // If nothing is running, invoke directly
        let mut running = self.running.lock().unwrap();
        if !*running {
            *running = true;
            drop(running); // Release lock before invoking

            let result = self.inner.invoke(request);

            *self.running.lock().unwrap() = false;

            // Process any queued items
            if !self.queue.lock().unwrap().is_empty() {
                *self.running.lock().unwrap() = true;
                self.process_queue();
            }

            return result;
        }

        // Something is running, queue this request
        drop(running); // Release lock
        self.queue.lock().unwrap().push_back(request.clone());

        // For now, return an error indicating the request was queued
        // In a real implementation, we'd use channels to return the result asynchronously
        Err(ClaudeError::InvocationFailed(
            "Request queued - async processing not yet implemented".to_string(),
        ))
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::claude_cli::types::PromptTask;

    /// Mock invoker for testing
    pub struct MockClaudeInvoker {
        pub should_succeed: bool,
        pub response_content: String,
        pub delay_ms: u64,
    }

    impl ClaudeInvoker for MockClaudeInvoker {
        fn invoke(&self, request: ClaudeRequest) -> Result<ClaudeResponse, ClaudeError> {
            if self.delay_ms > 0 {
                thread::sleep(Duration::from_millis(self.delay_ms));
            }

            if self.should_succeed {
                Ok(ClaudeResponse {
                    content: self.response_content.clone(),
                    task: request.task,
                    bug_id: request.bug_id,
                })
            } else {
                Err(ClaudeError::InvocationFailed("Mock failure".to_string()))
            }
        }
    }

    #[test]
    fn test_mock_invoker_success() {
        let invoker = MockClaudeInvoker {
            should_succeed: true,
            response_content: "Test response".to_string(),
            delay_ms: 0,
        };

        let request = ClaudeRequest::new_text("test prompt".to_string(), PromptTask::DescribeBug);
        let result = invoker.invoke(request);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.content, "Test response");
    }

    #[test]
    fn test_mock_invoker_failure() {
        let invoker = MockClaudeInvoker {
            should_succeed: false,
            response_content: "".to_string(),
            delay_ms: 0,
        };

        let request = ClaudeRequest::new_text("test prompt".to_string(), PromptTask::DescribeBug);
        let result = invoker.invoke(request);

        assert!(result.is_err());
    }

    #[test]
    fn test_queued_invoker_direct_when_idle() {
        let mock = Arc::new(MockClaudeInvoker {
            should_succeed: true,
            response_content: "Direct response".to_string(),
            delay_ms: 0,
        });

        let queued = QueuedClaudeInvoker::new(mock);
        let request = ClaudeRequest::new_text("test".to_string(), PromptTask::DescribeBug);

        let result = queued.invoke(request);
        assert!(result.is_ok());
    }
}
