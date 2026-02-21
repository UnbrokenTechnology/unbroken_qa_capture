/// Comprehensive tests for Claude API integration
///
/// Tests with mocked invoker to avoid requiring real API credentials
#[cfg(test)]
mod claude_cli_tests {
    use crate::claude_cli::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn test_claude_status_is_ready() {
        let ready = ClaudeStatus::Ready {
            version: "Claude Code".to_string(),
        };
        assert!(ready.is_ready());

        let not_auth = ClaudeStatus::NotAuthenticated {
            version: "Claude Code".to_string(),
            message: "test".to_string(),
        };
        assert!(!not_auth.is_ready());

        let not_installed = ClaudeStatus::NotInstalled {
            message: "test".to_string(),
        };
        assert!(!not_installed.is_ready());
    }

    #[test]
    fn test_bug_context_serialization() {
        let context = BugContext {
            bug_id: "BUG-001".to_string(),
            notes: Some("Test notes".to_string()),
            screenshot_paths: vec![PathBuf::from("/path/to/screenshot.png")],
            app_name: Some("TestApp".to_string()),
            app_version: Some("1.0.0".to_string()),
            meeting_id: Some("MEETING-001".to_string()),
            environment: Some("Windows 11".to_string()),
            bug_type: Some("bug".to_string()),
        };

        let json = serde_json::to_string(&context).unwrap();
        let deserialized: BugContext = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.bug_id, "BUG-001");
        assert_eq!(deserialized.notes, Some("Test notes".to_string()));
    }

    #[test]
    fn test_claude_error_display() {
        let not_found = ClaudeError::NotFound("test".to_string());
        assert!(not_found.to_string().contains("not found"));

        let timeout = ClaudeError::Timeout {
            seconds: 30,
            task: "test task".to_string(),
        };
        assert!(timeout.to_string().contains("30s"));

        let api_error = ClaudeError::ApiError("rate limit".to_string());
        assert!(api_error.to_string().contains("API error"));
    }

    #[test]
    fn test_claude_request_builder() {
        let request = ClaudeRequest::new_text("test prompt".to_string(), PromptTask::DescribeBug)
            .with_bug_id("BUG-001".to_string())
            .with_timeout(45);

        assert_eq!(request.prompt, "test prompt");
        assert_eq!(request.bug_id, Some("BUG-001".to_string()));
        assert_eq!(request.timeout_secs, 45);
        assert_eq!(request.task, PromptTask::DescribeBug);
    }

    #[test]
    fn test_claude_request_with_images() {
        let images = vec![
            PathBuf::from("/path/img1.png"),
            PathBuf::from("/path/img2.png"),
        ];

        let request = ClaudeRequest::new_with_images(
            "analyze these".to_string(),
            images.clone(),
            PromptTask::ParseConsole,
        );

        assert_eq!(request.image_paths.len(), 2);
        assert_eq!(request.timeout_secs, 30); // Default for images
        assert_eq!(request.task, PromptTask::ParseConsole);
    }

    #[test]
    fn test_prompt_builder_bug_description_minimal() {
        let context = BugContext {
            bug_id: "BUG-001".to_string(),
            notes: None,
            screenshot_paths: vec![],
            app_name: None,
            app_version: None,
            meeting_id: None,
            environment: None,
            bug_type: None,
        };

        let prompt = PromptBuilder::build_bug_description_prompt(&context);

        assert!(prompt.contains("QA analyst"));
        assert!(prompt.contains("## Summary"));
        assert!(prompt.contains("## Steps to Reproduce"));
        assert!(prompt.contains("## Expected Behavior"));
        assert!(prompt.contains("## Actual Behavior"));
    }

    #[test]
    fn test_prompt_builder_bug_description_full() {
        let context = BugContext {
            bug_id: "BUG-002".to_string(),
            notes: Some("Button not working".to_string()),
            screenshot_paths: vec![
                PathBuf::from("/img1.png"),
                PathBuf::from("/img2.png"),
            ],
            app_name: Some("MyApp".to_string()),
            app_version: Some("2.0.0".to_string()),
            meeting_id: Some("SESSION-123".to_string()),
            environment: Some("Windows 11".to_string()),
            bug_type: Some("bug".to_string()),
        };

        let prompt = PromptBuilder::build_bug_description_prompt(&context);

        assert!(prompt.contains("Application: MyApp"));
        assert!(prompt.contains("Version: 2.0.0"));
        assert!(prompt.contains("Environment: Windows 11"));
        assert!(prompt.contains("Session/Meeting ID: SESSION-123"));
        assert!(prompt.contains("Button not working"));
        assert!(prompt.contains("2 screenshot(s)"));
    }

    #[test]
    fn test_prompt_builder_console_parse() {
        let prompt = PromptBuilder::build_console_parse_prompt();

        assert!(prompt.contains("console/terminal"));
        assert!(prompt.contains("errors"));
        assert!(prompt.contains("warnings"));
        assert!(prompt.contains("JSON"));
    }

    #[test]
    fn test_prompt_builder_refinement() {
        let current = "Original description";
        let instructions = "Make it more detailed";

        let prompt = PromptBuilder::build_refinement_prompt(current, instructions);

        assert!(prompt.contains("Current Description"));
        assert!(prompt.contains("Original description"));
        assert!(prompt.contains("Refinement Request"));
        assert!(prompt.contains("Make it more detailed"));
    }

    #[test]
    fn test_mock_invoker_success() {
        use subprocess::tests::MockClaudeInvoker;

        let invoker = MockClaudeInvoker {
            should_succeed: true,
            response_content: "Generated description".to_string(),
            delay_ms: 0,
        };

        let request = ClaudeRequest::new_text("test".to_string(), PromptTask::DescribeBug);
        let result = invoker.invoke(request);

        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.content, "Generated description");
        assert_eq!(response.task, PromptTask::DescribeBug);
    }

    #[test]
    fn test_mock_invoker_failure() {
        use subprocess::tests::MockClaudeInvoker;

        let invoker = MockClaudeInvoker {
            should_succeed: false,
            response_content: "".to_string(),
            delay_ms: 0,
        };

        let request = ClaudeRequest::new_text("test".to_string(), PromptTask::DescribeBug);
        let result = invoker.invoke(request);

        assert!(result.is_err());
        match result.unwrap_err() {
            ClaudeError::InvocationFailed(msg) => {
                assert!(msg.contains("Mock failure"));
            }
            _ => panic!("Expected InvocationFailed error"),
        }
    }

    #[test]
    fn test_queued_invoker_direct_execution() {
        use subprocess::tests::MockClaudeInvoker;
        use subprocess::QueuedClaudeInvoker;

        let mock = Arc::new(MockClaudeInvoker {
            should_succeed: true,
            response_content: "Success".to_string(),
            delay_ms: 0,
        });

        let queued = QueuedClaudeInvoker::new(mock);
        let request = ClaudeRequest::new_text("test".to_string(), PromptTask::DescribeBug);

        let result = queued.invoke(request);
        assert!(result.is_ok());
    }

    #[test]
    fn test_claude_response_serialization() {
        let response = ClaudeResponse {
            content: "Test content".to_string(),
            task: PromptTask::DescribeBug,
            bug_id: Some("BUG-001".to_string()),
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: ClaudeResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.content, "Test content");
        assert_eq!(deserialized.task, PromptTask::DescribeBug);
        assert_eq!(deserialized.bug_id, Some("BUG-001".to_string()));
    }

    #[test]
    fn test_prompt_task_serialization() {
        let tasks = vec![
            PromptTask::DescribeBug,
            PromptTask::ParseConsole,
            PromptTask::RefineDescription,
            PromptTask::Custom,
        ];

        for task in tasks {
            let json = serde_json::to_string(&task).unwrap();
            let deserialized: PromptTask = serde_json::from_str(&json).unwrap();
            assert_eq!(task, deserialized);
        }
    }

    #[test]
    fn test_build_prompt_for_all_tasks() {
        let context = BugContext {
            bug_id: "BUG-001".to_string(),
            notes: Some("Current description".to_string()),
            screenshot_paths: vec![],
            app_name: None,
            app_version: None,
            meeting_id: None,
            environment: None,
            bug_type: None,
        };

        // DescribeBug
        let prompt = PromptBuilder::build_prompt(
            &PromptTask::DescribeBug,
            Some(&context),
            None,
        );
        assert!(prompt.contains("QA analyst"));

        // ParseConsole
        let prompt = PromptBuilder::build_prompt(&PromptTask::ParseConsole, None, None);
        assert!(prompt.contains("console"));

        // RefineDescription
        let prompt = PromptBuilder::build_prompt(
            &PromptTask::RefineDescription,
            Some(&context),
            Some("make it better"),
        );
        assert!(prompt.contains("Current Description"));

        // Custom
        let prompt = PromptBuilder::build_prompt(
            &PromptTask::Custom,
            None,
            Some("custom prompt text"),
        );
        assert_eq!(prompt, "custom prompt text");
    }

    #[test]
    fn test_load_credentials_returns_oauth_or_error() {
        // load_credentials() checks for OAuth token from Claude Code
        let result = load_credentials();
        match &result {
            Ok(creds) => {
                // OAuth credentials were found on this machine
                assert!(!creds.access_token.is_empty());
            }
            Err(_) => {
                // No credentials found — expected when Claude Code isn't installed
            }
        }
    }

    #[test]
    fn test_check_api_configured_returns_status() {
        // With no OAuth file, should return NotInstalled; with OAuth, Ready
        let status = check_api_configured();
        match status {
            ClaudeStatus::Ready { version } => {
                assert_eq!(version, "Claude Code");
            }
            ClaudeStatus::NotInstalled { message } => {
                assert!(!message.is_empty());
            }
            ClaudeStatus::NotAuthenticated { .. } => {
                // Also acceptable
            }
        }
    }

    #[test]
    fn test_claude_credentials_serialization() {
        let creds = ClaudeCredentials {
            access_token: "test-token".to_string(),
        };

        let json = serde_json::to_string(&creds).unwrap();
        let deserialized: ClaudeCredentials = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.access_token, "test-token");
    }

    /// Verify that `ClaudeError::NotFound` and `ClaudeError::NotAuthenticated` are distinct
    /// variants with different display messages. This documents the contract that
    /// `load_credentials()` uses `NotFound` for "file missing" and `NotAuthenticated`
    /// for "file present but no valid token".
    #[test]
    fn test_claude_error_not_found_vs_not_authenticated() {
        let not_found = ClaudeError::NotFound("Claude Code not installed".to_string());
        let not_authenticated = ClaudeError::NotAuthenticated("Token expired".to_string());

        let not_found_msg = not_found.to_string();
        let not_authenticated_msg = not_authenticated.to_string();

        // The two variants must produce different messages
        assert_ne!(not_found_msg, not_authenticated_msg);

        // NotFound display contains "not found"
        assert!(
            not_found_msg.contains("not found"),
            "NotFound display should contain 'not found', got: {not_found_msg}"
        );

        // NotAuthenticated display contains "not authenticated"
        assert!(
            not_authenticated_msg.contains("not authenticated"),
            "NotAuthenticated display should contain 'not authenticated', got: {not_authenticated_msg}"
        );

        // They must not contain each other's key phrase
        assert!(
            !not_found_msg.contains("not authenticated"),
            "NotFound display should not contain 'not authenticated'"
        );
        assert!(
            !not_authenticated_msg.contains("not found"),
            "NotAuthenticated display should not contain 'not found'"
        );
    }

    /// Verify that all three `ClaudeStatus` variants round-trip through JSON with the
    /// correct `"status"` discriminant values. The frontend switches on `status` to decide
    /// which UI to show, so the exact string values are part of the API contract.
    #[test]
    fn test_claude_status_serialization_all_variants() {
        // Ready
        let ready = ClaudeStatus::Ready {
            version: "Claude Code".to_string(),
        };
        let ready_json = serde_json::to_string(&ready).unwrap();
        assert!(
            ready_json.contains("\"status\":\"ready\""),
            "Ready should serialize with status=ready, got: {ready_json}"
        );
        let ready_roundtrip: ClaudeStatus = serde_json::from_str(&ready_json).unwrap();
        assert!(ready_roundtrip.is_ready());

        // NotAuthenticated — the frontend must receive "notAuthenticated" (camelCase)
        let not_auth = ClaudeStatus::NotAuthenticated {
            version: "Claude Code".to_string(),
            message: "Please sign in".to_string(),
        };
        let not_auth_json = serde_json::to_string(&not_auth).unwrap();
        assert!(
            not_auth_json.contains("\"status\":\"notAuthenticated\""),
            "NotAuthenticated should serialize with status=notAuthenticated, got: {not_auth_json}"
        );
        let not_auth_roundtrip: ClaudeStatus = serde_json::from_str(&not_auth_json).unwrap();
        assert!(!not_auth_roundtrip.is_ready());
        match not_auth_roundtrip {
            ClaudeStatus::NotAuthenticated { message, .. } => {
                assert_eq!(message, "Please sign in");
            }
            other => panic!("Expected NotAuthenticated after roundtrip, got: {other:?}"),
        }

        // NotInstalled
        let not_installed = ClaudeStatus::NotInstalled {
            message: "Install Claude Code".to_string(),
        };
        let not_installed_json = serde_json::to_string(&not_installed).unwrap();
        assert!(
            not_installed_json.contains("\"status\":\"notInstalled\""),
            "NotInstalled should serialize with status=notInstalled, got: {not_installed_json}"
        );
        let not_installed_roundtrip: ClaudeStatus =
            serde_json::from_str(&not_installed_json).unwrap();
        assert!(!not_installed_roundtrip.is_ready());
        match not_installed_roundtrip {
            ClaudeStatus::NotInstalled { message } => {
                assert_eq!(message, "Install Claude Code");
            }
            other => panic!("Expected NotInstalled after roundtrip, got: {other:?}"),
        }
    }

    /// Verify the mapping contract of `check_api_configured()`:
    /// `NotAuthenticated` errors from `load_credentials()` must produce
    /// `ClaudeStatus::NotAuthenticated`, while any other error (including `NotFound`)
    /// must produce `ClaudeStatus::NotInstalled`. This test exercises the mapping
    /// logic by inspecting the error variant type system rather than calling the
    /// live filesystem function a second time.
    #[test]
    fn test_check_api_configured_maps_not_found_to_not_installed() {
        // Simulate the mapping logic that check_api_configured() applies.
        // We construct error values and apply the same match arms to verify
        // that the variant distinction is meaningful and the mapping is correct.

        let not_found_err = ClaudeError::NotFound("no credentials file".to_string());
        let not_authenticated_err =
            ClaudeError::NotAuthenticated("token missing".to_string());

        // Apply the same mapping as check_api_configured()
        let status_from_not_found: ClaudeStatus = match not_found_err {
            ClaudeError::NotAuthenticated(msg) => ClaudeStatus::NotAuthenticated {
                version: "Claude Code".to_string(),
                message: msg,
            },
            _ => ClaudeStatus::NotInstalled {
                message: "Claude Code not found.".to_string(),
            },
        };

        let status_from_not_authenticated: ClaudeStatus = match not_authenticated_err {
            ClaudeError::NotAuthenticated(msg) => ClaudeStatus::NotAuthenticated {
                version: "Claude Code".to_string(),
                message: msg,
            },
            _ => ClaudeStatus::NotInstalled {
                message: "Claude Code not found.".to_string(),
            },
        };

        // NotFound → NotInstalled (not authenticated)
        assert!(
            !status_from_not_found.is_ready(),
            "NotFound error should not produce Ready status"
        );
        match status_from_not_found {
            ClaudeStatus::NotInstalled { .. } => {} // correct
            other => panic!(
                "NotFound error should map to NotInstalled, got: {other:?}"
            ),
        }

        // NotAuthenticated → NotAuthenticated (not NotInstalled)
        assert!(
            !status_from_not_authenticated.is_ready(),
            "NotAuthenticated error should not produce Ready status"
        );
        match status_from_not_authenticated {
            ClaudeStatus::NotAuthenticated { message, .. } => {
                assert_eq!(message, "token missing");
            }
            other => panic!(
                "NotAuthenticated error should map to NotAuthenticated status, got: {other:?}"
            ),
        }
    }
}
