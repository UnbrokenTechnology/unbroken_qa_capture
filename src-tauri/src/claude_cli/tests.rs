/// Comprehensive tests for Claude CLI integration
///
/// Tests with mocked subprocess to avoid requiring Claude CLI to be installed
#[cfg(test)]
mod claude_cli_tests {
    use crate::claude_cli::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn test_claude_status_is_ready() {
        let ready = ClaudeStatus::Ready {
            version: "1.0.0".to_string(),
        };
        assert!(ready.is_ready());

        let not_auth = ClaudeStatus::NotAuthenticated {
            version: "1.0.0".to_string(),
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
}
