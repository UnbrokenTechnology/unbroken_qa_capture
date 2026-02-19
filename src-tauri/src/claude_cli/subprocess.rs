//! Anthropic Messages API client
//!
//! Handles:
//! - Building multimodal API requests (text + images)
//! - Authentication via API key or OAuth token
//! - Response parsing
//! - Timeout enforcement
//! - Queue management (max 1 concurrent request)

use super::types::{ClaudeCredentials, ClaudeError, ClaudeRequest, ClaudeResponse, TokenSource};
use base64::Engine;
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

/// Trait for invoking the Anthropic API (enables mocking in tests)
pub trait ClaudeInvoker: Send + Sync {
    fn invoke(&self, request: ClaudeRequest) -> Result<ClaudeResponse, ClaudeError>;
}

/// Real implementation that calls the Anthropic Messages API via HTTP
pub struct RealClaudeInvoker {
    /// Pre-loaded credentials — set at construction time from settings + OAuth fallback
    credentials: ClaudeCredentials,
}

impl RealClaudeInvoker {
    pub fn new(credentials: ClaudeCredentials) -> Self {
        Self { credentials }
    }

    /// Call the Anthropic Messages API
    fn call_anthropic_api(&self, request: &ClaudeRequest) -> Result<ClaudeResponse, ClaudeError> {
        let client = reqwest::blocking::Client::builder()
            .timeout(Duration::from_secs(request.timeout_secs))
            .build()
            .map_err(|e| ClaudeError::ApiError(format!("Failed to create HTTP client: {}", e)))?;

        // Build messages content array (images + text)
        let mut content = Vec::new();

        // Add images as base64-encoded content blocks
        for image_path in &request.image_paths {
            let bytes = std::fs::read(image_path)
                .map_err(|e| ClaudeError::InvocationFailed(format!(
                    "Failed to read image {}: {}", image_path.display(), e
                )))?;
            let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);

            let ext = image_path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("png")
                .to_lowercase();
            let media_type = match ext.as_str() {
                "jpg" | "jpeg" => "image/jpeg",
                "gif" => "image/gif",
                "webp" => "image/webp",
                _ => "image/png", // default to PNG
            };

            content.push(serde_json::json!({
                "type": "image",
                "source": {
                    "type": "base64",
                    "media_type": media_type,
                    "data": b64
                }
            }));
        }

        // Add text prompt
        content.push(serde_json::json!({
            "type": "text",
            "text": request.prompt
        }));

        let body = serde_json::json!({
            "model": "claude-sonnet-4-20250514",
            "max_tokens": 4096,
            "messages": [{
                "role": "user",
                "content": content
            }]
        });

        // Build the request with auth headers
        let mut req_builder = client
            .post("https://api.anthropic.com/v1/messages")
            .header("content-type", "application/json")
            .header("anthropic-version", "2023-06-01");

        match self.credentials.token_source {
            TokenSource::ApiKey => {
                req_builder = req_builder.header("x-api-key", &self.credentials.access_token);
            }
            TokenSource::OAuthToken => {
                req_builder = req_builder.header(
                    "Authorization",
                    format!("Bearer {}", self.credentials.access_token),
                );
            }
        }

        let response = req_builder
            .json(&body)
            .send()
            .map_err(|e| {
                if e.is_timeout() {
                    ClaudeError::Timeout {
                        seconds: request.timeout_secs,
                        task: format!("{:?}", request.task),
                    }
                } else {
                    ClaudeError::ApiError(format!("HTTP request failed: {}", e))
                }
            })?;

        // Check HTTP status
        let status = response.status();
        let resp_text = response
            .text()
            .map_err(|e| ClaudeError::ApiError(format!("Failed to read response body: {}", e)))?;

        if !status.is_success() {
            if status.as_u16() == 401 {
                return Err(ClaudeError::NotAuthenticated(
                    "Invalid or expired API credentials. Check your API key.".to_string(),
                ));
            }
            if status.as_u16() == 429 {
                return Err(ClaudeError::ApiError(
                    "Rate limit exceeded. Please wait and try again.".to_string(),
                ));
            }
            return Err(ClaudeError::ApiError(format!(
                "HTTP {}: {}",
                status, resp_text
            )));
        }

        // Parse Messages API response: { "content": [{ "type": "text", "text": "..." }] }
        let resp_json: serde_json::Value = serde_json::from_str(&resp_text)
            .map_err(|e| ClaudeError::ParseError(format!("Invalid JSON response: {}", e)))?;

        let text = resp_json
            .get("content")
            .and_then(|c| c.as_array())
            .and_then(|arr| arr.first())
            .and_then(|block| block.get("text"))
            .and_then(|t| t.as_str())
            .ok_or_else(|| {
                ClaudeError::ParseError(format!(
                    "Unexpected response structure: {}",
                    &resp_text[..resp_text.len().min(200)]
                ))
            })?;

        Ok(ClaudeResponse {
            content: text.to_string(),
            task: request.task.clone(),
            bug_id: request.bug_id.clone(),
        })
    }
}

impl ClaudeInvoker for RealClaudeInvoker {
    fn invoke(&self, request: ClaudeRequest) -> Result<ClaudeResponse, ClaudeError> {
        self.call_anthropic_api(&request)
    }
}

/// Queued invoker that ensures max 1 concurrent request
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
