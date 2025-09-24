pub mod license;
pub mod policy;
pub mod output;
pub mod exceptions;
pub mod config;

// Re-export main types for easy access
pub use license::{PackageLicense, LicenseReport, LicenseSummary, LicenseTypes};
pub use policy::{LicensePolicy, ViolationLevel, Violation, ViolationSummary};
