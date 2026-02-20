//! Prompt construction for Claude CLI tasks
//!
//! This module builds focused prompts for different AI tasks:
//! - Bug description generation
//! - Console screenshot parsing
//! - Description refinement
//! - Capture-to-bug assignment suggestion

use super::types::{BugContext, PromptTask};

/// Summary of a bug used in capture assignment prompts
pub struct BugSummary {
    pub display_id: String,
    pub title: Option<String>,
    pub notes: Option<String>,
    /// True if this bug's first screenshot was included as an image in the request
    pub has_reference_image: bool,
    /// 1-based index of the reference image in the content array (if included)
    pub reference_image_index: Option<usize>,
}

pub struct PromptBuilder;

impl PromptBuilder {
    /// Build a prompt for bug description generation
    pub fn build_bug_description_prompt(context: &BugContext) -> String {
        let mut prompt = String::new();

        prompt.push_str("You are a QA analyst writing a structured bug report. ");
        prompt.push_str("Based on the provided screenshots and context, generate a clear, actionable bug description.\n\n");

        // Add context metadata
        if let Some(app_name) = &context.app_name {
            prompt.push_str(&format!("Application: {}\n", app_name));
        }
        if let Some(version) = &context.app_version {
            prompt.push_str(&format!("Version: {}\n", version));
        }
        if let Some(env) = &context.environment {
            prompt.push_str(&format!("Environment: {}\n", env));
        }
        if let Some(meeting_id) = &context.meeting_id {
            prompt.push_str(&format!("Session/Meeting ID: {}\n", meeting_id));
        }

        prompt.push('\n');

        // Add notes if present
        if let Some(notes) = &context.notes {
            if !notes.trim().is_empty() {
                prompt.push_str(&format!("Tester's Notes:\n{}\n\n", notes));
            }
        }

        // Add screenshot count
        let screenshot_count = context.screenshot_paths.len();
        if screenshot_count > 0 {
            prompt.push_str(&format!(
                "{} screenshot(s) are attached showing the issue.\n\n",
                screenshot_count
            ));
        }

        // Request structured output
        prompt.push_str("Please provide a bug report in the following format:\n\n");
        prompt.push_str("## Summary\n");
        prompt.push_str("[One-sentence description of the issue]\n\n");
        prompt.push_str("## Steps to Reproduce\n");
        prompt.push_str("1. [Step 1]\n");
        prompt.push_str("2. [Step 2]\n");
        prompt.push_str("...\n\n");
        prompt.push_str("## Expected Behavior\n");
        prompt.push_str("[What should happen]\n\n");
        prompt.push_str("## Actual Behavior\n");
        prompt.push_str("[What actually happens]\n\n");
        prompt.push_str("## Additional Context\n");
        prompt.push_str("[Any other relevant details from the screenshots or notes]\n\n");

        // Add bug type if specified
        if let Some(bug_type) = &context.bug_type {
            if bug_type != "bug" {
                prompt.push_str(&format!("Note: This is categorized as a '{}', not a bug. Adjust the format as appropriate.\n", bug_type));
            }
        }

        prompt
    }

    /// Build a prompt for console screenshot parsing
    pub fn build_console_parse_prompt() -> String {
        let mut prompt = String::new();

        prompt.push_str("You are analyzing a console/terminal screenshot. ");
        prompt.push_str("Extract all errors, warnings, and important log messages from the image.\n\n");

        prompt.push_str("Please provide the output in the following JSON format:\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"errors\": [\"error message 1\", \"error message 2\"],\n");
        prompt.push_str("  \"warnings\": [\"warning message 1\", \"warning message 2\"],\n");
        prompt.push_str("  \"logs\": [\"important log 1\", \"important log 2\"]\n");
        prompt.push_str("}\n\n");

        prompt.push_str("If no errors/warnings are found, return empty arrays. ");
        prompt.push_str("Focus on technical details: error codes, stack traces, file paths, line numbers.\n");

        prompt
    }

    /// Build a prompt for description refinement
    pub fn build_refinement_prompt(
        current_description: &str,
        refinement_instructions: &str,
    ) -> String {
        let mut prompt = String::new();

        prompt.push_str("You are refining an existing bug report based on user feedback.\n\n");

        prompt.push_str("Current Description:\n");
        prompt.push_str("---\n");
        prompt.push_str(current_description);
        prompt.push_str("\n---\n\n");

        prompt.push_str("User's Refinement Request:\n");
        prompt.push_str(refinement_instructions);
        prompt.push_str("\n\n");

        prompt.push_str("Please provide the updated bug report, incorporating the requested changes while preserving the overall structure and any accurate details.\n");

        prompt
    }

    /// Build a prompt for AI capture-to-bug assignment.
    ///
    /// The unsorted screenshot is always image #1 in the content array.
    /// Bug reference images (if any) follow, and `BugSummary::reference_image_index`
    /// records their 1-based position so the prompt can refer to them.
    pub fn build_capture_assignment_prompt(bugs: &[BugSummary]) -> String {
        let mut prompt = String::new();

        prompt.push_str("You are a QA assistant. The FIRST image attached to this message is an unsorted screenshot captured during a testing session. ");
        prompt.push_str("Your task is to determine which existing bug this screenshot most likely belongs to, or whether it represents a new, previously unreported issue.\n\n");

        if bugs.is_empty() {
            prompt.push_str("There are NO existing bugs in this session yet. This screenshot likely represents a new issue.\n\n");
        } else {
            prompt.push_str("Here are the existing bugs in this session:\n\n");
            for bug in bugs {
                prompt.push_str(&format!("### {}\n", bug.display_id));
                if let Some(title) = &bug.title {
                    prompt.push_str(&format!("Title: {}\n", title));
                }
                if let Some(notes) = &bug.notes {
                    if !notes.trim().is_empty() {
                        prompt.push_str(&format!("Notes: {}\n", notes));
                    }
                }
                if bug.has_reference_image {
                    if let Some(idx) = bug.reference_image_index {
                        prompt.push_str(&format!(
                            "(A reference screenshot for this bug is attached as image #{})\n",
                            idx
                        ));
                    }
                }
                prompt.push('\n');
            }
        }

        prompt.push_str("Based on visual similarity, context, and any textual clues, respond with ONLY a JSON object (no markdown fences, no explanation outside the JSON):\n\n");
        prompt.push_str("{\n");
        prompt.push_str("  \"bug_display_id\": \"BUG-001\" or null,\n");
        prompt.push_str("  \"confidence\": 0.0 to 1.0,\n");
        prompt.push_str("  \"reasoning\": \"Brief explanation of why this screenshot matches (or doesn't match) an existing bug.\"\n");
        prompt.push_str("}\n\n");
        prompt.push_str("Set bug_display_id to null if the screenshot appears to be a new issue not covered by any existing bug.\n");
        prompt.push_str("Set confidence to 0.0 if you cannot determine a match at all.\n");

        prompt
    }

    /// Build a custom prompt (user-provided)
    pub fn build_custom_prompt(user_prompt: &str) -> String {
        user_prompt.to_string()
    }

    /// Build prompt based on task type
    pub fn build_prompt(
        task: &PromptTask,
        context: Option<&BugContext>,
        custom_text: Option<&str>,
    ) -> String {
        match task {
            PromptTask::DescribeBug => {
                if let Some(ctx) = context {
                    Self::build_bug_description_prompt(ctx)
                } else {
                    "Generate a bug description.".to_string()
                }
            }
            PromptTask::ParseConsole => Self::build_console_parse_prompt(),
            PromptTask::RefineDescription => {
                if let (Some(ctx), Some(instructions)) = (context, custom_text) {
                    // Assume context.notes contains the current description
                    let current_desc = ctx.notes.as_deref().unwrap_or("");
                    Self::build_refinement_prompt(current_desc, instructions)
                } else {
                    "Refine the description.".to_string()
                }
            }
            PromptTask::Custom => {
                if let Some(text) = custom_text {
                    Self::build_custom_prompt(text)
                } else {
                    "".to_string()
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_build_bug_description_prompt_minimal() {
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
    fn test_build_bug_description_prompt_full() {
        let context = BugContext {
            bug_id: "BUG-002".to_string(),
            notes: Some("Button doesn't respond to clicks".to_string()),
            screenshot_paths: vec![
                PathBuf::from("/path/to/screenshot1.png"),
                PathBuf::from("/path/to/screenshot2.png"),
            ],
            app_name: Some("TestApp".to_string()),
            app_version: Some("1.2.3".to_string()),
            meeting_id: Some("SESSION-001".to_string()),
            environment: Some("Windows 11".to_string()),
            bug_type: Some("bug".to_string()),
        };

        let prompt = PromptBuilder::build_bug_description_prompt(&context);

        assert!(prompt.contains("Application: TestApp"));
        assert!(prompt.contains("Version: 1.2.3"));
        assert!(prompt.contains("Environment: Windows 11"));
        assert!(prompt.contains("Session/Meeting ID: SESSION-001"));
        assert!(prompt.contains("Button doesn't respond to clicks"));
        assert!(prompt.contains("2 screenshot(s)"));
    }

    #[test]
    fn test_build_console_parse_prompt() {
        let prompt = PromptBuilder::build_console_parse_prompt();

        assert!(prompt.contains("console/terminal screenshot"));
        assert!(prompt.contains("errors"));
        assert!(prompt.contains("warnings"));
        assert!(prompt.contains("JSON format"));
    }

    #[test]
    fn test_build_refinement_prompt() {
        let current = "This is the current bug description.";
        let instructions = "Make it more detailed.";

        let prompt = PromptBuilder::build_refinement_prompt(current, instructions);

        assert!(prompt.contains("Current Description"));
        assert!(prompt.contains(current));
        assert!(prompt.contains("Refinement Request"));
        assert!(prompt.contains(instructions));
    }

    #[test]
    fn test_build_custom_prompt() {
        let custom = "My custom prompt text";
        let prompt = PromptBuilder::build_custom_prompt(custom);

        assert_eq!(prompt, custom);
    }

    #[test]
    fn test_build_prompt_describe_bug() {
        let context = BugContext {
            bug_id: "BUG-003".to_string(),
            notes: Some("Test note".to_string()),
            screenshot_paths: vec![],
            app_name: Some("App".to_string()),
            app_version: None,
            meeting_id: None,
            environment: None,
            bug_type: None,
        };

        let prompt = PromptBuilder::build_prompt(
            &PromptTask::DescribeBug,
            Some(&context),
            None,
        );

        assert!(prompt.contains("QA analyst"));
        assert!(prompt.contains("Test note"));
    }

    #[test]
    fn test_build_prompt_parse_console() {
        let prompt = PromptBuilder::build_prompt(&PromptTask::ParseConsole, None, None);

        assert!(prompt.contains("console/terminal"));
    }

    #[test]
    fn test_build_prompt_custom() {
        let custom_text = "Custom prompt";
        let prompt = PromptBuilder::build_prompt(&PromptTask::Custom, None, Some(custom_text));

        assert_eq!(prompt, custom_text);
    }

    #[test]
    fn test_build_capture_assignment_prompt_no_bugs() {
        let prompt = PromptBuilder::build_capture_assignment_prompt(&[]);

        assert!(prompt.contains("unsorted screenshot"));
        assert!(prompt.contains("NO existing bugs"));
        assert!(prompt.contains("bug_display_id"));
        assert!(prompt.contains("confidence"));
        assert!(prompt.contains("reasoning"));
    }

    #[test]
    fn test_build_capture_assignment_prompt_with_bugs() {
        let bugs = vec![
            BugSummary {
                display_id: "BUG-001".to_string(),
                title: Some("Login button broken".to_string()),
                notes: Some("Can't click login".to_string()),
                has_reference_image: true,
                reference_image_index: Some(2),
            },
            BugSummary {
                display_id: "BUG-002".to_string(),
                title: None,
                notes: None,
                has_reference_image: false,
                reference_image_index: None,
            },
        ];

        let prompt = PromptBuilder::build_capture_assignment_prompt(&bugs);

        assert!(prompt.contains("BUG-001"));
        assert!(prompt.contains("Login button broken"));
        assert!(prompt.contains("Can't click login"));
        assert!(prompt.contains("image #2"));
        assert!(prompt.contains("BUG-002"));
        assert!(!prompt.contains("NO existing bugs"));
    }
}
