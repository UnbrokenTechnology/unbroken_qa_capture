use super::types::QaProfile;
use std::sync::Mutex;

/// Trait defining profile CRUD operations
#[allow(dead_code)]
pub trait ProfileRepository: Send + Sync {
    fn create(&self, profile: &QaProfile) -> Result<(), String>;
    fn get(&self, id: &str) -> Result<Option<QaProfile>, String>;
    fn list(&self) -> Result<Vec<QaProfile>, String>;
    fn update(&self, profile: &QaProfile) -> Result<(), String>;
    fn delete(&self, id: &str) -> Result<(), String>;
}

/// In-memory profile repository — used in tests and as a reference implementation
#[allow(dead_code)]
pub struct InMemoryProfileRepository {
    profiles: Mutex<Vec<QaProfile>>,
}

impl InMemoryProfileRepository {
    #[allow(dead_code)]
    pub fn new() -> Self {
        InMemoryProfileRepository {
            profiles: Mutex::new(Vec::new()),
        }
    }
}

impl ProfileRepository for InMemoryProfileRepository {
    fn create(&self, profile: &QaProfile) -> Result<(), String> {
        let mut profiles = self.profiles.lock().unwrap();
        if profiles.iter().any(|p| p.id == profile.id) {
            return Err(format!("Profile with id '{}' already exists", profile.id));
        }
        profiles.push(profile.clone());
        Ok(())
    }

    fn get(&self, id: &str) -> Result<Option<QaProfile>, String> {
        let profiles = self.profiles.lock().unwrap();
        Ok(profiles.iter().find(|p| p.id == id).cloned())
    }

    fn list(&self) -> Result<Vec<QaProfile>, String> {
        let profiles = self.profiles.lock().unwrap();
        Ok(profiles.clone())
    }

    fn update(&self, profile: &QaProfile) -> Result<(), String> {
        let mut profiles = self.profiles.lock().unwrap();
        match profiles.iter_mut().find(|p| p.id == profile.id) {
            Some(existing) => {
                *existing = profile.clone();
                Ok(())
            }
            None => Err(format!("Profile with id '{}' not found", profile.id)),
        }
    }

    fn delete(&self, id: &str) -> Result<(), String> {
        let mut profiles = self.profiles.lock().unwrap();
        let original_len = profiles.len();
        profiles.retain(|p| p.id != id);
        if profiles.len() == original_len {
            return Err(format!("Profile with id '{}' not found", id));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::types::{AreaCategory, CustomFieldType, CustomMetadataField, QaProfile};

    fn make_profile(id: &str, name: &str) -> QaProfile {
        QaProfile {
            id: id.to_string(),
            name: name.to_string(),
            linear_config: None,
            area_categories: vec![AreaCategory {
                code: "UI".to_string(),
                name: "User Interface".to_string(),
                description: None,
            }],
            custom_fields: vec![CustomMetadataField {
                key: "severity".to_string(),
                label: "Severity".to_string(),
                field_type: CustomFieldType::Select,
                default_value: Some("medium".to_string()),
                required: false,
            }],
            title_conventions: None,
            created_at: "2024-01-01T00:00:00Z".to_string(),
            updated_at: "2024-01-01T00:00:00Z".to_string(),
        }
    }

    #[test]
    fn test_create_and_get() {
        let repo = InMemoryProfileRepository::new();
        let profile = make_profile("profile-1", "My Profile");

        repo.create(&profile).unwrap();

        let retrieved = repo.get("profile-1").unwrap();
        assert!(retrieved.is_some());
        let retrieved_profile = retrieved.unwrap();
        assert_eq!(retrieved_profile.id, "profile-1");
        assert_eq!(retrieved_profile.name, "My Profile");
    }

    #[test]
    fn test_get_returns_none_for_missing_id() {
        let repo = InMemoryProfileRepository::new();
        let result = repo.get("nonexistent").unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_list_profiles() {
        let repo = InMemoryProfileRepository::new();
        repo.create(&make_profile("profile-1", "Alpha")).unwrap();
        repo.create(&make_profile("profile-2", "Beta")).unwrap();

        let profiles = repo.list().unwrap();
        assert_eq!(profiles.len(), 2);
        assert_eq!(profiles[0].id, "profile-1");
        assert_eq!(profiles[1].id, "profile-2");
    }

    #[test]
    fn test_list_empty_repository() {
        let repo = InMemoryProfileRepository::new();
        let profiles = repo.list().unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_update_profile() {
        let repo = InMemoryProfileRepository::new();
        let profile = make_profile("profile-1", "Original Name");
        repo.create(&profile).unwrap();

        let mut updated = profile.clone();
        updated.name = "Updated Name".to_string();
        repo.update(&updated).unwrap();

        let retrieved = repo.get("profile-1").unwrap().unwrap();
        assert_eq!(retrieved.name, "Updated Name");
    }

    #[test]
    fn test_update_nonexistent_profile_fails() {
        let repo = InMemoryProfileRepository::new();
        let profile = make_profile("profile-99", "Ghost");
        let result = repo.update(&profile);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_delete_profile() {
        let repo = InMemoryProfileRepository::new();
        let profile = make_profile("profile-1", "To Delete");
        repo.create(&profile).unwrap();

        repo.delete("profile-1").unwrap();

        let retrieved = repo.get("profile-1").unwrap();
        assert!(retrieved.is_none());

        let profiles = repo.list().unwrap();
        assert!(profiles.is_empty());
    }

    #[test]
    fn test_delete_nonexistent_profile_fails() {
        let repo = InMemoryProfileRepository::new();
        let result = repo.delete("nonexistent");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[test]
    fn test_create_duplicate_id_fails() {
        let repo = InMemoryProfileRepository::new();
        let profile = make_profile("profile-1", "First");
        repo.create(&profile).unwrap();

        let duplicate = make_profile("profile-1", "Second");
        let result = repo.create(&duplicate);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("already exists"));

        // Original is unchanged
        let retrieved = repo.get("profile-1").unwrap().unwrap();
        assert_eq!(retrieved.name, "First");
    }
}
