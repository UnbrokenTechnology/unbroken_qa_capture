use rusqlite::Connection;

use super::repository::{ProfileRepository, SqliteProfileRepository};
use super::types::{
    AreaCategory, CustomFieldType, CustomMetadataField, LinearProfileConfig, QaProfile,
    TitleConventions,
};

/// Deterministic ID for the built-in Contio MeetingOS profile.
/// Using a fixed string ensures the profile is only ever seeded once,
/// even if the user deletes and reinitialises the database.
pub const CONTIO_PROFILE_ID: &str = "contio-meetingos-default-profile";

/// Seed the default "Contio MeetingOS" profile if no profiles exist yet.
///
/// This is called once on every app startup, immediately after the database
/// schema is initialised.  The operation is a no-op when the database already
/// contains at least one profile, so it is safe to call repeatedly.
pub fn seed_default_profile(conn: &Connection) -> Result<(), String> {
    let repo = SqliteProfileRepository::new(conn);

    let existing = repo.list()?;
    if !existing.is_empty() {
        return Ok(());
    }

    let profile = build_contio_profile();
    repo.create(&profile)?;

    // Persist the active-profile-id setting so the UI picks it up immediately.
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value, updated_at)
         VALUES ('active_profile_id', ?1, datetime('now'))",
        rusqlite::params![CONTIO_PROFILE_ID],
    )
    .map_err(|e| format!("Failed to set active profile: {}", e))?;

    Ok(())
}

/// Build the Contio MeetingOS QaProfile from the values that were previously
/// documented in LINEAR_INTEGRATION.md.
fn build_contio_profile() -> QaProfile {
    let now = chrono::Utc::now().to_rfc3339();

    QaProfile {
        id: CONTIO_PROFILE_ID.to_string(),
        name: "Contio MeetingOS".to_string(),

        linear_config: Some(LinearProfileConfig {
            team_id: "44c86ac8-cb80-4302-9d81-a0a350b2c352".to_string(),
            // API key is left blank; the user supplies it in Settings.
            api_key: String::new(),
            // Bug default: assigned to Adam Clark
            default_assignee_id: Some("976c10e6-d43b-49f0-9007-e59c88c4624b".to_string()),
            // Bug default labels: untriaged
            default_bug_label_ids: vec![
                "9ac15a53-d87b-4f6c-8360-1870482689f7".to_string(), // untriaged
            ],
            // Feature-request default labels: Feature-request + untriaged
            default_feature_label_ids: vec![
                "90a8f314-886d-4952-ad6c-0fbedf136c43".to_string(), // Feature-request
                "9ac15a53-d87b-4f6c-8360-1870482689f7".to_string(), // untriaged
            ],
            // Default state: Backlog
            default_state_id: Some("aa635d13-f2bb-48f2-a395-2fd15e0b0441".to_string()),
        }),

        area_categories: vec![
            AreaCategory {
                code: "LGN".to_string(),
                name: "Login".to_string(),
                description: Some("Email and Social Logins".to_string()),
            },
            AreaCategory {
                code: "ASET".to_string(),
                name: "Application Settings".to_string(),
                description: Some(
                    "Settings that change how MeetingOS works and are controlled by an administrator or owner"
                        .to_string(),
                ),
            },
            AreaCategory {
                code: "USET".to_string(),
                name: "User Settings".to_string(),
                description: Some(
                    "Settings set by an individual user for their own MeetingOS account"
                        .to_string(),
                ),
            },
            AreaCategory {
                code: "TPP".to_string(),
                name: "Transcript Processing".to_string(),
                description: Some(
                    "Processing meeting transcripts either uploaded or post meeting".to_string(),
                ),
            },
            AreaCategory {
                code: "INSTALL".to_string(),
                name: "Desktop Installation".to_string(),
                description: Some(
                    "Covers tests during the installation process of the desktop software including download, installation and uninstall"
                        .to_string(),
                ),
            },
            AreaCategory {
                code: "VIEWS".to_string(),
                name: "Meeting Views".to_string(),
                description: Some(
                    "Ways to view past and future meetings (Home, In My Workspace, Drafts, Recaps on Hold, Backlog, Search)"
                        .to_string(),
                ),
            },
            AreaCategory {
                code: "DRAFT".to_string(),
                name: "Backlog and Drafts".to_string(),
                description: Some(
                    "Meeting state before a meeting is officially created which includes Backlog items and Meeting Drafts"
                        .to_string(),
                ),
            },
            AreaCategory {
                code: "AGENDA".to_string(),
                name: "Agenda Creation".to_string(),
                description: Some(
                    "Creating, editing, and updating the agenda either manually or using contio.ai"
                        .to_string(),
                ),
            },
            AreaCategory {
                code: "MEET".to_string(),
                name: "Meeting Capture".to_string(),
                description: Some("Meeting capture".to_string()),
            },
            AreaCategory {
                code: "SUMRY".to_string(),
                name: "Meeting Summary".to_string(),
                description: Some("Meeting Summary creation and editing".to_string()),
            },
            AreaCategory {
                code: "SHARE".to_string(),
                name: "Sharing Summaries".to_string(),
                description: Some(
                    "Sharing the meeting summary via copy and paste, email, URL".to_string(),
                ),
            },
            AreaCategory {
                code: "ACTION".to_string(),
                name: "Action Items".to_string(),
                description: Some(
                    "Action item creation and viewing - anything related to action items specifically"
                        .to_string(),
                ),
            },
            AreaCategory {
                code: "CAL".to_string(),
                name: "Calendar Integration".to_string(),
                description: Some(
                    "Calendar integrations and calendar event connections".to_string(),
                ),
            },
            AreaCategory {
                code: "META".to_string(),
                name: "Meeting Metadata".to_string(),
                description: Some(
                    "Meeting time, date, title, and other metadata entered during setup is shows and saves accurately for the user"
                        .to_string(),
                ),
            },
            AreaCategory {
                code: "AI".to_string(),
                name: "Contio AI".to_string(),
                description: Some("Contio AI features and related functionality".to_string()),
            },
        ],

        custom_fields: vec![
            CustomMetadataField {
                key: "meeting_id".to_string(),
                label: "Meeting ID".to_string(),
                field_type: CustomFieldType::Text,
                default_value: None,
                required: true,
                options: None,
            },
            CustomMetadataField {
                key: "software_version".to_string(),
                label: "Contio MeetingOS Version".to_string(),
                field_type: CustomFieldType::Text,
                default_value: None,
                required: false,
                options: None,
            },
            CustomMetadataField {
                key: "impact".to_string(),
                label: "Impact Level".to_string(),
                field_type: CustomFieldType::Select,
                default_value: None,
                required: true,
                options: Some(vec![
                    "Critical".to_string(),
                    "High".to_string(),
                    "Medium".to_string(),
                    "Low".to_string(),
                ]),
            },
            CustomMetadataField {
                key: "workaround".to_string(),
                label: "Workaround Available".to_string(),
                field_type: CustomFieldType::Select,
                default_value: None,
                required: false,
                options: Some(vec![
                    "Yes".to_string(),
                    "No".to_string(),
                    "Partial".to_string(),
                ]),
            },
            CustomMetadataField {
                key: "area_category".to_string(),
                label: "Area / Category".to_string(),
                field_type: CustomFieldType::Select,
                default_value: None,
                required: false,
                // Options are empty here — the UI populates them at runtime
                // from the profile's `area_categories` list.
                options: Some(vec![]),
            },
        ],

        title_conventions: Some(TitleConventions {
            bug_prefix: "QA Bug Found - ".to_string(),
            feature_prefix: "[QA Feature Suggestion] ".to_string(),
        }),

        created_at: now.clone(),
        updated_at: now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database;

    fn open_test_db() -> rusqlite::Connection {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        database::init_database(&conn).unwrap();
        conn
    }

    #[test]
    fn test_seed_creates_profile_when_empty() {
        let conn = open_test_db();
        seed_default_profile(&conn).unwrap();

        let repo = SqliteProfileRepository::new(&conn);
        let profiles = repo.list().unwrap();
        assert_eq!(profiles.len(), 1);

        let profile = &profiles[0];
        assert_eq!(profile.id, CONTIO_PROFILE_ID);
        assert_eq!(profile.name, "Contio MeetingOS");
    }

    #[test]
    fn test_seed_is_noop_when_profiles_exist() {
        let conn = open_test_db();

        // Seed once
        seed_default_profile(&conn).unwrap();

        // Seed again — should remain at 1 profile
        seed_default_profile(&conn).unwrap();

        let repo = SqliteProfileRepository::new(&conn);
        let profiles = repo.list().unwrap();
        assert_eq!(profiles.len(), 1);
    }

    #[test]
    fn test_seed_does_not_overwrite_existing_profiles() {
        let conn = open_test_db();

        // Create a user profile first
        let repo = SqliteProfileRepository::new(&conn);
        let user_profile = QaProfile {
            id: "user-profile-1".to_string(),
            name: "My Custom Profile".to_string(),
            linear_config: None,
            area_categories: vec![],
            custom_fields: vec![],
            title_conventions: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        };
        repo.create(&user_profile).unwrap();

        // Seed should be a no-op
        seed_default_profile(&conn).unwrap();

        let profiles = repo.list().unwrap();
        assert_eq!(profiles.len(), 1);
        assert_eq!(profiles[0].id, "user-profile-1");
    }

    #[test]
    fn test_seed_sets_active_profile_id() {
        let conn = open_test_db();
        seed_default_profile(&conn).unwrap();

        let active: Option<String> = conn
            .query_row(
                "SELECT value FROM settings WHERE key = 'active_profile_id'",
                [],
                |row| row.get(0),
            )
            .ok();

        assert_eq!(active, Some(CONTIO_PROFILE_ID.to_string()));
    }

    #[test]
    fn test_contio_profile_has_all_area_categories() {
        let profile = build_contio_profile();
        // LINEAR_INTEGRATION.md had 15 area categories
        assert_eq!(profile.area_categories.len(), 15);
    }

    #[test]
    fn test_contio_profile_linear_config() {
        let profile = build_contio_profile();
        let cfg = profile.linear_config.unwrap();
        assert_eq!(cfg.team_id, "44c86ac8-cb80-4302-9d81-a0a350b2c352");
        assert_eq!(cfg.default_bug_label_ids.len(), 1);
        assert_eq!(cfg.default_feature_label_ids.len(), 2);
        assert_eq!(
            cfg.default_state_id,
            Some("aa635d13-f2bb-48f2-a395-2fd15e0b0441".to_string())
        );
    }

    #[test]
    fn test_contio_profile_custom_fields() {
        let profile = build_contio_profile();
        assert_eq!(profile.custom_fields.len(), 5);

        let meeting_id = profile.custom_fields.iter().find(|f| f.key == "meeting_id").unwrap();
        assert!(meeting_id.required);
        assert_eq!(meeting_id.field_type, CustomFieldType::Text);
        assert!(meeting_id.options.is_none());

        let version = profile
            .custom_fields
            .iter()
            .find(|f| f.key == "software_version")
            .unwrap();
        assert!(!version.required);
        assert_eq!(version.field_type, CustomFieldType::Text);
        assert!(version.options.is_none());

        let impact = profile.custom_fields.iter().find(|f| f.key == "impact").unwrap();
        assert!(impact.required);
        assert_eq!(impact.field_type, CustomFieldType::Select);
        let impact_opts = impact.options.as_ref().unwrap();
        assert_eq!(impact_opts, &["Critical", "High", "Medium", "Low"]);

        let workaround =
            profile.custom_fields.iter().find(|f| f.key == "workaround").unwrap();
        assert!(!workaround.required);
        assert_eq!(workaround.field_type, CustomFieldType::Select);
        let workaround_opts = workaround.options.as_ref().unwrap();
        assert_eq!(workaround_opts, &["Yes", "No", "Partial"]);

        let area_cat =
            profile.custom_fields.iter().find(|f| f.key == "area_category").unwrap();
        assert!(!area_cat.required);
        assert_eq!(area_cat.field_type, CustomFieldType::Select);
        let area_opts = area_cat.options.as_ref().unwrap();
        assert!(area_opts.is_empty(), "area_category options should be empty (populated at runtime)");
    }

    #[test]
    fn test_contio_profile_title_conventions() {
        let profile = build_contio_profile();
        let tc = profile.title_conventions.unwrap();
        assert_eq!(tc.bug_prefix, "QA Bug Found - ");
        assert_eq!(tc.feature_prefix, "[QA Feature Suggestion] ");
    }
}
