pub mod models;
pub mod storage;
pub mod checker;
pub mod interactive;

// Re-export commonly used items
pub use models::{Exception, ExceptionsFile};
pub use storage::{load_exceptions, save_exceptions, get_exceptions_file_path};
pub use interactive::{prompt_for_exception, handle_interactive_exceptions};

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use chrono::{Utc, NaiveDate};

    #[test]
    fn test_exception_creation() {
        let exception = Exception {
            name: "test-package".to_string(),
            version: Some("1.0.0".to_string()),
            reason: "testing".to_string(),
            added_by: Some("user".to_string()),
            added_date: Utc::now(),
            expires: Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
            permanent: false,
            added_interactively: true,
        };
        
        assert_eq!(exception.name, "test-package");
        assert_eq!(exception.version, Some("1.0.0".to_string()));
        assert!(!exception.permanent);
    }

    #[test]
    fn test_exceptions_file_add_exception() {
        let mut exceptions_file = ExceptionsFile::new();
        assert_eq!(exceptions_file.exceptions.len(), 0);
        
        let exception = Exception {
            name: "test-package".to_string(),
            version: None,
            reason: "testing".to_string(),
            added_by: None,
            added_date: Utc::now(),
            expires: None,
            permanent: true,
            added_interactively: false,
        };
        
        exceptions_file.add_exception(exception);
        assert_eq!(exceptions_file.exceptions.len(), 1);
    }

    #[test]
    fn test_is_excepted_exact_match() {
        let mut exceptions_file = ExceptionsFile::new();
        let exception = Exception {
            name: "test-package".to_string(),
            version: Some("1.0.0".to_string()),
            reason: "testing".to_string(),
            added_by: None,
            added_date: Utc::now(),
            expires: None,
            permanent: true,
            added_interactively: false,
        };
        
        exceptions_file.add_exception(exception);
        
        // Exact match should be excepted
        assert!(exceptions_file.is_excepted("test-package", Some("1.0.0")));
        
        // Different version should not be excepted
        assert!(!exceptions_file.is_excepted("test-package", Some("2.0.0")));
        
        // Different package should not be excepted
        assert!(!exceptions_file.is_excepted("other-package", Some("1.0.0")));
    }

    #[test]
    fn test_is_excepted_wildcard_version() {
        let mut exceptions_file = ExceptionsFile::new();
        let exception = Exception {
            name: "test-package".to_string(),
            version: Some("*".to_string()),
            reason: "testing".to_string(),
            added_by: None,
            added_date: Utc::now(),
            expires: None,
            permanent: true,
            added_interactively: false,
        };
        
        exceptions_file.add_exception(exception);
        
        // Any version should be excepted with wildcard
        assert!(exceptions_file.is_excepted("test-package", Some("1.0.0")));
        assert!(exceptions_file.is_excepted("test-package", Some("2.0.0")));
    }

    #[test]
    fn test_is_excepted_no_version() {
        let mut exceptions_file = ExceptionsFile::new();
        let exception = Exception {
            name: "test-package".to_string(),
            version: None,
            reason: "testing".to_string(),
            added_by: None,
            added_date: Utc::now(),
            expires: None,
            permanent: true,
            added_interactively: false,
        };
        
        exceptions_file.add_exception(exception);
        
        // Should match regardless of version when exception has no version
        assert!(exceptions_file.is_excepted("test-package", Some("1.0.0")));
        assert!(exceptions_file.is_excepted("test-package", None));
    }

    #[test]
    fn test_cleanup_expired() {
        let mut exceptions_file = ExceptionsFile::new();
        
        // Add expired exception
        let expired_exception = Exception {
            name: "expired-package".to_string(),
            version: None,
            reason: "testing".to_string(),
            added_by: None,
            added_date: Utc::now(),
            expires: Some(NaiveDate::from_ymd_opt(2020, 1, 1).unwrap()), // Past date
            permanent: false,
            added_interactively: false,
        };
        
        // Add valid exception
        let valid_exception = Exception {
            name: "valid-package".to_string(),
            version: None,
            reason: "testing".to_string(),
            added_by: None,
            added_date: Utc::now(),
            expires: Some(NaiveDate::from_ymd_opt(2030, 1, 1).unwrap()), // Future date
            permanent: false,
            added_interactively: false,
        };
        
        exceptions_file.add_exception(expired_exception);
        exceptions_file.add_exception(valid_exception);
        
        assert_eq!(exceptions_file.exceptions.len(), 2);
        
        let removed_count = exceptions_file.cleanup_expired();
        assert_eq!(removed_count, 1);
        assert_eq!(exceptions_file.exceptions.len(), 1);
        assert_eq!(exceptions_file.exceptions[0].name, "valid-package");
    }

    #[test]
    fn test_file_save_and_load() {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("test_exceptions.toml");
        
        let mut exceptions_file = ExceptionsFile::new();
        let exception = Exception {
            name: "test-package".to_string(),
            version: Some("1.0.0".to_string()),
            reason: "testing".to_string(),
            added_by: Some("user".to_string()),
            added_date: Utc::now(),
            expires: None,
            permanent: true,
            added_interactively: false,
        };
        
        exceptions_file.add_exception(exception);
        
        // Save to file
        exceptions_file.save_to_file(&file_path).unwrap();
        assert!(file_path.exists());
        
        // Load from file
        let loaded_exceptions = ExceptionsFile::load_from_file(&file_path).unwrap();
        assert_eq!(loaded_exceptions.exceptions.len(), 1);
        assert_eq!(loaded_exceptions.exceptions[0].name, "test-package");
        assert_eq!(loaded_exceptions.exceptions[0].version, Some("1.0.0".to_string()));
    }
}
