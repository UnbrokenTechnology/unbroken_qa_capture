use super::*;
use std::sync::{Arc, RwLock};

/// Mock ticketing integration for testing
struct MockTicketingIntegration {
    authenticated: Arc<RwLock<bool>>,
    should_fail: Arc<RwLock<bool>>,
    created_tickets: Arc<RwLock<Vec<CreateTicketRequest>>>,
}

impl MockTicketingIntegration {
    fn new() -> Self {
        Self {
            authenticated: Arc::new(RwLock::new(false)),
            should_fail: Arc::new(RwLock::new(false)),
            created_tickets: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn set_should_fail(&self, fail: bool) {
        *self.should_fail.write().unwrap() = fail;
    }

    fn get_created_tickets(&self) -> Vec<CreateTicketRequest> {
        self.created_tickets.read().unwrap().clone()
    }
}

impl TicketingIntegration for MockTicketingIntegration {
    fn authenticate(&self, credentials: &TicketingCredentials) -> TicketingResult<()> {
        if *self.should_fail.read().unwrap() {
            return Err(TicketingError::AuthenticationFailed(
                "Mock authentication failed".to_string(),
            ));
        }

        // Simple validation: API key must not be empty
        if credentials.api_key.is_empty() {
            return Err(TicketingError::AuthenticationFailed(
                "API key cannot be empty".to_string(),
            ));
        }

        *self.authenticated.write().unwrap() = true;
        Ok(())
    }

    fn create_ticket(&self, request: &CreateTicketRequest) -> TicketingResult<CreateTicketResponse> {
        if !*self.authenticated.read().unwrap() {
            return Err(TicketingError::AuthenticationFailed(
                "Not authenticated".to_string(),
            ));
        }

        if *self.should_fail.read().unwrap() {
            return Err(TicketingError::CreationFailed(
                "Mock creation failed".to_string(),
            ));
        }

        // Store the request
        self.created_tickets.write().unwrap().push(request.clone());

        Ok(CreateTicketResponse {
            id: "mock-id-123".to_string(),
            url: "https://mock.example.com/issue/MOCK-123".to_string(),
            identifier: "MOCK-123".to_string(),
        })
    }

    fn check_connection(&self) -> TicketingResult<ConnectionStatus> {
        if *self.should_fail.read().unwrap() {
            return Ok(ConnectionStatus {
                connected: false,
                message: Some("Connection failed".to_string()),
                integration_name: "Mock".to_string(),
            });
        }

        Ok(ConnectionStatus {
            connected: *self.authenticated.read().unwrap(),
            message: None,
            integration_name: "Mock".to_string(),
        })
    }

    fn name(&self) -> &str {
        "Mock"
    }
}

#[test]
fn test_mock_integration_authentication() {
    let integration = MockTicketingIntegration::new();

    let credentials = TicketingCredentials {
        api_key: "test-api-key".to_string(),
        workspace_id: None,
        team_id: None,
    };

    let result = integration.authenticate(&credentials);
    assert!(result.is_ok());
}

#[test]
fn test_mock_integration_authentication_empty_key() {
    let integration = MockTicketingIntegration::new();

    let credentials = TicketingCredentials {
        api_key: "".to_string(),
        workspace_id: None,
        team_id: None,
    };

    let result = integration.authenticate(&credentials);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TicketingError::AuthenticationFailed(_)
    ));
}

#[test]
fn test_mock_integration_create_ticket_success() {
    let integration = MockTicketingIntegration::new();

    // Authenticate first
    let credentials = TicketingCredentials {
        api_key: "test-api-key".to_string(),
        workspace_id: None,
        team_id: None,
    };
    integration.authenticate(&credentials).unwrap();

    // Create ticket
    let request = CreateTicketRequest {
        title: "Test Bug".to_string(),
        description: "This is a test bug".to_string(),
        attachments: vec![],
        priority: Some("1".to_string()),
        labels: vec![],
    };

    let result = integration.create_ticket(&request);
    assert!(result.is_ok());

    let response = result.unwrap();
    assert_eq!(response.id, "mock-id-123");
    assert_eq!(response.identifier, "MOCK-123");
    assert!(response.url.contains("MOCK-123"));
}

#[test]
fn test_mock_integration_create_ticket_not_authenticated() {
    let integration = MockTicketingIntegration::new();

    let request = CreateTicketRequest {
        title: "Test Bug".to_string(),
        description: "This is a test bug".to_string(),
        attachments: vec![],
        priority: None,
        labels: vec![],
    };

    let result = integration.create_ticket(&request);
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TicketingError::AuthenticationFailed(_)
    ));
}

#[test]
fn test_mock_integration_create_ticket_with_attachments() {
    let integration = MockTicketingIntegration::new();

    // Authenticate
    let credentials = TicketingCredentials {
        api_key: "test-api-key".to_string(),
        workspace_id: None,
        team_id: None,
    };
    integration.authenticate(&credentials).unwrap();

    // Create ticket with attachments
    let request = CreateTicketRequest {
        title: "Bug with Screenshots".to_string(),
        description: "Bug description".to_string(),
        attachments: vec![
            "/path/to/screenshot1.png".to_string(),
            "/path/to/screenshot2.png".to_string(),
        ],
        priority: Some("2".to_string()),
        labels: vec!["bug".to_string(), "ui".to_string()],
    };

    let result = integration.create_ticket(&request);
    assert!(result.is_ok());

    // Verify the request was stored
    let tickets = integration.get_created_tickets();
    assert_eq!(tickets.len(), 1);
    assert_eq!(tickets[0].attachments.len(), 2);
    assert_eq!(tickets[0].labels.len(), 2);
}

#[test]
fn test_mock_integration_check_connection_authenticated() {
    let integration = MockTicketingIntegration::new();

    // Before authentication
    let status = integration.check_connection().unwrap();
    assert!(!status.connected);

    // After authentication
    let credentials = TicketingCredentials {
        api_key: "test-api-key".to_string(),
        workspace_id: None,
        team_id: None,
    };
    integration.authenticate(&credentials).unwrap();

    let status = integration.check_connection().unwrap();
    assert!(status.connected);
    assert_eq!(status.integration_name, "Mock");
}

#[test]
fn test_mock_integration_check_connection_failure() {
    let integration = MockTicketingIntegration::new();
    integration.set_should_fail(true);

    let status = integration.check_connection().unwrap();
    assert!(!status.connected);
    assert!(status.message.is_some());
}

#[test]
fn test_ticketing_error_display() {
    let err = TicketingError::AuthenticationFailed("Invalid API key".to_string());
    assert_eq!(err.to_string(), "Authentication failed: Invalid API key");

    let err = TicketingError::NetworkError("Timeout".to_string());
    assert_eq!(err.to_string(), "Network error: Timeout");

    let err = TicketingError::InvalidConfig("Missing team ID".to_string());
    assert_eq!(err.to_string(), "Invalid configuration: Missing team ID");

    let err = TicketingError::CreationFailed("GraphQL error".to_string());
    assert_eq!(err.to_string(), "Ticket creation failed: GraphQL error");

    let err = TicketingError::ConnectionFailed("Cannot reach server".to_string());
    assert_eq!(err.to_string(), "Connection check failed: Cannot reach server");
}

#[test]
fn test_linear_integration_creation() {
    let integration = LinearIntegration::new();
    assert_eq!(integration.name(), "Linear");
}

#[test]
fn test_linear_integration_check_connection_not_authenticated() {
    let integration = LinearIntegration::new();
    let status = integration.check_connection().unwrap();

    assert!(!status.connected);
    assert_eq!(status.integration_name, "Linear");
    assert!(status.message.is_some());
}

// Note: Full integration tests with actual Linear API would require
// real API keys and should be run separately in integration test suite
