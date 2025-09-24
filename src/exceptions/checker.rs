use chrono::Utc;
use super::models::ExceptionsFile;

impl ExceptionsFile {
    pub fn is_excepted(&self, package_name: &str, package_version: Option<&str>) -> bool {
        self.exceptions.iter().any(|exc| {
            // Check if exception has expired
            if let Some(expires) = exc.expires {
                if Utc::now().date_naive() > expires {
                    return false;
                }
            }

            // Check package name match
            if exc.name != package_name {
                return false;
            }

            // Check version match (if specified)
            match (&exc.version, package_version) {
                (Some(exc_version), Some(pkg_version)) => {
                    exc_version == "*" || exc_version == pkg_version
                }
                (Some(_), None) => false,
                (None, _) => true,
            }
        })
    }

    pub fn cleanup_expired(&mut self) -> usize {
        let original_count = self.exceptions.len();
        let today = Utc::now().date_naive();
        
        self.exceptions.retain(|exc| {
            match exc.expires {
                Some(expires) => today <= expires,
                None => true, // Keep permanent exceptions
            }
        });
        
        original_count - self.exceptions.len()
    }
}
