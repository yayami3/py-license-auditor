pub mod config;
pub mod matcher;
pub mod checker;

// Re-export main types
pub use config::{LicensePolicy, LicenseRule, PackageException};
pub use matcher::ViolationLevel;
pub use checker::{Violation, ViolationSummary};
