use serde::{Deserialize, Serialize};
use glob::Pattern;
use super::config::{LicenseRule, LicensePolicy, PackageException};

/// 違反レベル
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ViolationLevel {
    Allowed,
    ReviewRequired,
    Forbidden,
    Unknown,
}

impl LicenseRule {
    /// ライセンス名がこのルールにマッチするかチェック
    pub fn matches(&self, license: &str) -> bool {
        // 完全一致をチェック
        if self.exact.iter().any(|exact| exact == license) {
            return true;
        }
        
        // パターンマッチングをチェック
        for pattern_str in &self.patterns {
            if let Ok(pattern) = Pattern::new(pattern_str) {
                if pattern.matches(license) {
                    return true;
                }
            }
        }
        
        false
    }
    
    /// マッチしたルール（完全一致またはパターン）を返す
    pub fn find_match(&self, license: &str) -> Option<String> {
        // 完全一致をチェック
        for exact in &self.exact {
            if exact == license {
                return Some(format!("exact: {}", exact));
            }
        }
        
        // パターンマッチングをチェック
        for pattern_str in &self.patterns {
            if let Ok(pattern) = Pattern::new(pattern_str) {
                if pattern.matches(license) {
                    return Some(format!("pattern: {}", pattern_str));
                }
            }
        }
        
        None
    }
}

impl LicensePolicy {
    /// ライセンスの違反レベルをチェック
    pub fn check_license(&self, license: &str) -> ViolationLevel {
        // 禁止リストを最初にチェック（最も重要）
        if self.forbidden_licenses.matches(license) {
            return ViolationLevel::Forbidden;
        }
        
        // 許可リストをチェック
        if self.allowed_licenses.matches(license) {
            return ViolationLevel::Allowed;
        }
        
        // レビュー必要リストをチェック
        if self.review_required.matches(license) {
            return ViolationLevel::ReviewRequired;
        }
        
        // どのルールにもマッチしない場合は不明
        ViolationLevel::Unknown
    }
    
    /// パッケージが例外リストに含まれているかチェック
    pub fn is_exception(&self, package_name: &str, package_version: Option<&str>) -> Option<&PackageException> {
        self.exceptions.iter().find(|exception| {
            exception.name == package_name && 
            (exception.version.is_none() || 
             exception.version.as_deref() == package_version)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_rule_exact_match() {
        let rule = LicenseRule {
            exact: vec!["MIT".to_string(), "Apache-2.0".to_string()],
            patterns: vec![],
        };

        assert!(rule.matches("MIT"));
        assert!(rule.matches("Apache-2.0"));
        assert!(!rule.matches("GPL-3.0"));
    }

    #[test]
    fn test_license_rule_pattern_match() {
        let rule = LicenseRule {
            exact: vec![],
            patterns: vec!["GPL-*".to_string(), "BSD-*".to_string()],
        };

        assert!(rule.matches("GPL-3.0"));
        assert!(rule.matches("BSD-3-Clause"));
        assert!(!rule.matches("MIT"));
    }

    #[test]
    fn test_license_policy_check() {
        let policy = LicensePolicy {
            name: "Test Policy".to_string(),
            description: None,
            allowed_licenses: LicenseRule {
                exact: vec!["MIT".to_string()],
                patterns: vec![],
            },
            forbidden_licenses: LicenseRule {
                exact: vec!["GPL-3.0".to_string()],
                patterns: vec![],
            },
            review_required: LicenseRule {
                exact: vec!["Apache-2.0".to_string()],
                patterns: vec![],
            },
            exceptions: vec![],
        };

        assert_eq!(policy.check_license("MIT"), ViolationLevel::Allowed);
        assert_eq!(policy.check_license("GPL-3.0"), ViolationLevel::Forbidden);
        assert_eq!(policy.check_license("Apache-2.0"), ViolationLevel::ReviewRequired);
        assert_eq!(policy.check_license("Unknown"), ViolationLevel::Unknown);
    }
}
