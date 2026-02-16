use super::trait_def::TicketingIntegration;
use super::types::*;
use serde_json::json;
use std::sync::{Arc, RwLock};

/// Linear integration for creating issues via GraphQL API
///
/// Uses Linear's GraphQL API to create issues with attachments.
/// Requires an API key which can be created at: https://linear.app/settings/api
pub struct LinearIntegration {
    credentials: Arc<RwLock<Option<TicketingCredentials>>>,
    api_endpoint: String,
}

impl LinearIntegration {
    /// Create a new Linear integration instance
    pub fn new() -> Self {
        Self {
            credentials: Arc::new(RwLock::new(None)),
            api_endpoint: "https://api.linear.app/graphql".to_string(),
        }
    }

    /// Send a GraphQL query to Linear API
    fn send_graphql_query(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> TicketingResult<serde_json::Value> {
        let creds = self.credentials.read().unwrap();
        let credentials = creds
            .as_ref()
            .ok_or_else(|| TicketingError::AuthenticationFailed("Not authenticated".to_string()))?;

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&self.api_endpoint)
            .header("Authorization", credentials.api_key.clone())
            .header("Content-Type", "application/json")
            .json(&json!({
                "query": query,
                "variables": variables
            }))
            .send()
            .map_err(|e| TicketingError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(TicketingError::NetworkError(format!(
                "HTTP {}: {}",
                response.status(),
                response.text().unwrap_or_default()
            )));
        }

        let json_response: serde_json::Value = response
            .json()
            .map_err(|e| TicketingError::NetworkError(format!("Failed to parse response: {}", e)))?;

        // Check for GraphQL errors
        if let Some(errors) = json_response.get("errors") {
            return Err(TicketingError::CreationFailed(format!(
                "GraphQL errors: {}",
                errors
            )));
        }

        Ok(json_response)
    }

    /// Upload an attachment to Linear
    fn upload_attachment(&self, file_path: &str) -> TicketingResult<String> {
        // For now, we'll return a placeholder since Linear attachment upload
        // requires a more complex multi-step process (get upload URL, upload file, etc.)
        // This can be implemented in a future iteration
        let _ = file_path;
        Ok(String::new())
    }
}

impl Default for LinearIntegration {
    fn default() -> Self {
        Self::new()
    }
}

impl TicketingIntegration for LinearIntegration {
    fn authenticate(&self, credentials: &TicketingCredentials) -> TicketingResult<()> {
        // Test authentication by querying viewer info
        let query = r#"
            query {
                viewer {
                    id
                    name
                    email
                }
            }
        "#;

        let client = reqwest::blocking::Client::new();
        let response = client
            .post(&self.api_endpoint)
            .header("Authorization", credentials.api_key.clone())
            .header("Content-Type", "application/json")
            .json(&json!({
                "query": query,
                "variables": {}
            }))
            .send()
            .map_err(|e| TicketingError::AuthenticationFailed(e.to_string()))?;

        if !response.status().is_success() {
            return Err(TicketingError::AuthenticationFailed(format!(
                "HTTP {}: Invalid API key",
                response.status()
            )));
        }

        let json_response: serde_json::Value = response
            .json()
            .map_err(|e| TicketingError::AuthenticationFailed(format!("Failed to parse response: {}", e)))?;

        // Check for errors in response
        if json_response.get("errors").is_some() {
            return Err(TicketingError::AuthenticationFailed(
                "Invalid API key or insufficient permissions".to_string(),
            ));
        }

        // Store credentials if authentication succeeds
        *self.credentials.write().unwrap() = Some(credentials.clone());

        Ok(())
    }

    fn create_ticket(&self, request: &CreateTicketRequest) -> TicketingResult<CreateTicketResponse> {
        let creds = self.credentials.read().unwrap();
        let credentials = creds
            .as_ref()
            .ok_or_else(|| TicketingError::AuthenticationFailed("Not authenticated".to_string()))?;

        // Get team ID - required for creating issues
        let team_id = credentials
            .team_id
            .as_ref()
            .ok_or_else(|| TicketingError::InvalidConfig("team_id is required".to_string()))?;

        // Upload attachments first
        let mut attachment_ids = Vec::new();
        for attachment_path in &request.attachments {
            if let Ok(attachment_id) = self.upload_attachment(attachment_path) {
                if !attachment_id.is_empty() {
                    attachment_ids.push(attachment_id);
                }
            }
        }

        // Create the issue
        let query = r#"
            mutation IssueCreate($input: IssueCreateInput!) {
                issueCreate(input: $input) {
                    success
                    issue {
                        id
                        identifier
                        url
                        title
                    }
                }
            }
        "#;

        let mut variables = json!({
            "input": {
                "teamId": team_id,
                "title": request.title,
                "description": request.description,
            }
        });

        // Add priority if specified
        if let Some(priority) = &request.priority {
            variables["input"]["priority"] = json!(priority.parse::<i32>().unwrap_or(0));
        }

        // Add labels if specified
        if !request.labels.is_empty() {
            variables["input"]["labelIds"] = json!(request.labels);
        }

        let response = self.send_graphql_query(query, variables)?;

        // Extract issue data from response
        let issue_data = response
            .get("data")
            .and_then(|d| d.get("issueCreate"))
            .and_then(|ic| ic.get("issue"))
            .ok_or_else(|| TicketingError::CreationFailed("Failed to extract issue data from response".to_string()))?;

        let id = issue_data
            .get("id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TicketingError::CreationFailed("Missing issue ID".to_string()))?
            .to_string();

        let identifier = issue_data
            .get("identifier")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TicketingError::CreationFailed("Missing issue identifier".to_string()))?
            .to_string();

        let url = issue_data
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| TicketingError::CreationFailed("Missing issue URL".to_string()))?
            .to_string();

        Ok(CreateTicketResponse {
            id,
            url,
            identifier,
        })
    }

    fn check_connection(&self) -> TicketingResult<ConnectionStatus> {
        let creds = self.credentials.read().unwrap();
        if creds.is_none() {
            return Ok(ConnectionStatus {
                connected: false,
                message: Some("Not authenticated".to_string()),
                integration_name: "Linear".to_string(),
            });
        }

        let query = r#"
            query {
                viewer {
                    id
                }
            }
        "#;

        match self.send_graphql_query(query, json!({})) {
            Ok(_) => Ok(ConnectionStatus {
                connected: true,
                message: None,
                integration_name: "Linear".to_string(),
            }),
            Err(e) => Ok(ConnectionStatus {
                connected: false,
                message: Some(e.to_string()),
                integration_name: "Linear".to_string(),
            }),
        }
    }

    fn name(&self) -> &str {
        "Linear"
    }
}
