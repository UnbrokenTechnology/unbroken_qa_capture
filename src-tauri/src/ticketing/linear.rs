use super::trait_def::TicketingIntegration;
use super::types::*;
use serde_json::json;
use std::io::Read;
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

    /// Create a Linear integration instance with a custom API endpoint (for testing only)
    #[cfg(test)]
    pub(crate) fn with_endpoint(api_endpoint: &str) -> Self {
        Self {
            credentials: Arc::new(RwLock::new(None)),
            api_endpoint: api_endpoint.to_string(),
        }
    }

    /// Set credentials directly without network validation (for testing only)
    #[cfg(test)]
    pub(crate) fn set_credentials_for_test(&self, credentials: TicketingCredentials) {
        *self.credentials.write().unwrap() = Some(credentials);
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

    /// Upload a file to Linear using the three-step process:
    /// 1. Request a pre-signed upload URL via the `fileUpload` mutation
    /// 2. PUT the file bytes to the signed URL with the required headers
    /// 3. Return the permanent asset URL for embedding in the issue description
    ///
    /// Returns the asset URL on success, or an error if upload fails.
    fn upload_attachment(&self, file_path: &str) -> TicketingResult<String> {
        use std::path::Path;

        let path = Path::new(file_path);

        // Read file bytes
        let mut file = std::fs::File::open(path).map_err(|e| {
            TicketingError::NetworkError(format!("Cannot open file {}: {}", file_path, e))
        })?;
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes).map_err(|e| {
            TicketingError::NetworkError(format!("Cannot read file {}: {}", file_path, e))
        })?;

        // Determine MIME type from extension
        let content_type = match path.extension().and_then(|e| e.to_str()) {
            Some("png") => "image/png",
            Some("jpg") | Some("jpeg") => "image/jpeg",
            Some("gif") => "image/gif",
            Some("webp") => "image/webp",
            Some("mp4") => "video/mp4",
            Some("mov") => "video/quicktime",
            Some("webm") => "video/webm",
            _ => "application/octet-stream",
        };

        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("attachment");

        let file_size = file_bytes.len() as u64;

        // Step 1: Request a pre-signed upload URL from Linear
        let creds = self.credentials.read().unwrap();
        let credentials = creds
            .as_ref()
            .ok_or_else(|| TicketingError::AuthenticationFailed("Not authenticated".to_string()))?;

        let upload_query = r#"
            mutation FileUpload($contentType: String!, $filename: String!, $size: Int!) {
                fileUpload(contentType: $contentType, filename: $filename, size: $size) {
                    uploadFile {
                        uploadUrl
                        assetUrl
                        headers {
                            key
                            value
                        }
                    }
                }
            }
        "#;

        let upload_variables = json!({
            "contentType": content_type,
            "filename": file_name,
            "size": file_size
        });

        let client = reqwest::blocking::Client::new();
        let graphql_response = client
            .post(&self.api_endpoint)
            .header("Authorization", credentials.api_key.clone())
            .header("Content-Type", "application/json")
            .json(&json!({
                "query": upload_query,
                "variables": upload_variables
            }))
            .send()
            .map_err(|e| TicketingError::NetworkError(format!("fileUpload mutation failed: {}", e)))?;

        if !graphql_response.status().is_success() {
            return Err(TicketingError::NetworkError(format!(
                "fileUpload mutation HTTP {}: {}",
                graphql_response.status(),
                graphql_response.text().unwrap_or_default()
            )));
        }

        let graphql_json: serde_json::Value = graphql_response.json().map_err(|e| {
            TicketingError::NetworkError(format!("Failed to parse fileUpload response: {}", e))
        })?;

        if let Some(errors) = graphql_json.get("errors") {
            return Err(TicketingError::NetworkError(format!(
                "fileUpload GraphQL errors: {}",
                errors
            )));
        }

        let upload_file = graphql_json
            .get("data")
            .and_then(|d| d.get("fileUpload"))
            .and_then(|fu| fu.get("uploadFile"))
            .ok_or_else(|| {
                TicketingError::NetworkError(
                    "fileUpload mutation returned no uploadFile".to_string(),
                )
            })?;

        let upload_url = upload_file
            .get("uploadUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TicketingError::NetworkError("fileUpload returned no uploadUrl".to_string())
            })?;

        let asset_url = upload_file
            .get("assetUrl")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                TicketingError::NetworkError("fileUpload returned no assetUrl".to_string())
            })?
            .to_string();

        // Step 2: PUT file bytes to the signed S3 URL with required headers
        let mut put_request = client
            .put(upload_url)
            .header("Content-Type", content_type)
            .header("Cache-Control", "public, max-age=31536000");

        // Apply all auth headers returned by Linear
        if let Some(headers) = upload_file.get("headers").and_then(|h| h.as_array()) {
            for header in headers {
                let key = header.get("key").and_then(|k| k.as_str()).unwrap_or("");
                let value = header.get("value").and_then(|v| v.as_str()).unwrap_or("");
                if !key.is_empty() {
                    put_request = put_request.header(key, value);
                }
            }
        }

        let put_response = put_request
            .body(file_bytes)
            .send()
            .map_err(|e| TicketingError::NetworkError(format!("S3 PUT upload failed: {}", e)))?;

        if !put_response.status().is_success() {
            return Err(TicketingError::NetworkError(format!(
                "S3 PUT upload HTTP {}: {}",
                put_response.status(),
                put_response.text().unwrap_or_default()
            )));
        }

        // Step 3: Return the permanent asset URL for embedding in the description
        Ok(asset_url)
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

        // Upload attachments and collect asset URLs; log failures but continue
        let mut attachment_results: Vec<AttachmentUploadResult> = Vec::new();
        let mut asset_urls: Vec<String> = Vec::new();
        for attachment_path in &request.attachments {
            match self.upload_attachment(attachment_path) {
                Ok(url) if !url.is_empty() => {
                    attachment_results.push(AttachmentUploadResult {
                        file_path: attachment_path.clone(),
                        success: true,
                        message: url.clone(),
                    });
                    asset_urls.push(url);
                }
                Ok(_) => {
                    attachment_results.push(AttachmentUploadResult {
                        file_path: attachment_path.clone(),
                        success: false,
                        message: "Upload returned empty URL".to_string(),
                    });
                }
                Err(e) => {
                    attachment_results.push(AttachmentUploadResult {
                        file_path: attachment_path.clone(),
                        success: false,
                        message: e.to_string(),
                    });
                }
            }
        }

        // Build description with embedded screenshot images (markdown format)
        let mut full_description = request.description.clone();
        if !asset_urls.is_empty() {
            full_description.push_str("\n\n## Screenshots\n\n");
            for (i, url) in asset_urls.iter().enumerate() {
                full_description.push_str(&format!("![Screenshot {}]({})\n\n", i + 1, url));
            }
        }
        let upload_failures: Vec<&str> = attachment_results
            .iter()
            .filter(|r| !r.success)
            .map(|r| r.file_path.as_str())
            .collect();
        if !upload_failures.is_empty() {
            full_description.push_str("\n\n*Note: The following screenshots could not be uploaded: ");
            full_description.push_str(&upload_failures.join(", "));
            full_description.push('*');
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
                "description": full_description,
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

        // Add assignee if specified (from profile defaults)
        if let Some(assignee_id) = &request.assignee_id {
            variables["input"]["assigneeId"] = json!(assignee_id);
        }

        // Add workflow state if specified (from profile defaults)
        if let Some(state_id) = &request.state_id {
            variables["input"]["stateId"] = json!(state_id);
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
            attachment_results,
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
