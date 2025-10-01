use serde::{Deserialize, Serialize};
use super::matcher::ViolationLevel;
use super::config::LicensePolicy;
use crate::license::{PackageLicense, normalize_license_name};

/// 違反の詳細情報
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Violation {
    pub package_name: String,
    pub package_version: Option<String>,
    pub license: Option<String>,
    pub violation_level: ViolationLevel,
    pub matched_rule: Option<String>,
    pub message: String,
}

/// 違反のサマリー情報
#[derive(Debug, Serialize, Deserialize)]
pub struct ViolationSummary {
    pub total: usize,
    pub errors: usize,    // Forbidden
    pub warnings: usize,  // ReviewRequired + Unknown
    pub details: Vec<Violation>,
}

impl LicensePolicy {
    /// パッケージリストから違反を検出
    pub fn detect_violations(&self, packages: &[PackageLicense]) -> ViolationSummary {
        let mut violations = Vec::new();
        
        for package in packages {
            // 例外チェック
            if self.is_exception(&package.name, package.version.as_deref()).is_some() {
                continue; // 例外なのでスキップ
            }
            
            // ライセンスがない場合
            let license = match &package.license {
                Some(license) if !license.trim().is_empty() => license,
                _ => {
                    violations.push(Violation {
                        package_name: package.name.clone(),
                        package_version: package.version.clone(),
                        license: None,
                        violation_level: ViolationLevel::Unknown,
                        matched_rule: None,
                        message: "No license information found".to_string(),
                    });
                    continue;
                }
            };
            
            // ライセンス名を正規化
            let normalized_license = normalize_license_name(license);
            
            // 違反レベルをチェック
            let violation_level = self.check_license(&normalized_license);
            
            // Allowedでない場合は違反として記録
            if violation_level != ViolationLevel::Allowed {
                let matched_rule = match violation_level {
                    ViolationLevel::Forbidden => self.forbidden_licenses.find_match(&normalized_license),
                    ViolationLevel::ReviewRequired => self.review_required.find_match(&normalized_license),
                    _ => None,
                };
                
                let message = match violation_level {
                    ViolationLevel::Forbidden => format!("License '{}' is forbidden by policy", normalized_license),
                    ViolationLevel::ReviewRequired => format!("License '{}' requires review", normalized_license),
                    ViolationLevel::Unknown => format!("License '{}' is not in allowed list", normalized_license),
                    ViolationLevel::Allowed => unreachable!(),
                };
                
                violations.push(Violation {
                    package_name: package.name.clone(),
                    package_version: package.version.clone(),
                    license: Some(normalized_license),
                    violation_level,
                    matched_rule,
                    message,
                });
            }
        }
        
        // サマリーを計算
        let errors = violations.iter().filter(|v| v.violation_level == ViolationLevel::Forbidden).count();
        let warnings = violations.iter().filter(|v| 
            v.violation_level == ViolationLevel::ReviewRequired || 
            v.violation_level == ViolationLevel::Unknown
        ).count();
        
        ViolationSummary {
            total: violations.len(),
            errors,
            warnings,
            details: violations,
        }
    }
}
