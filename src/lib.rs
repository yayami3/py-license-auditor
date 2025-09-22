pub mod license;
pub mod policy;
pub mod output;

// Re-export main types for easy access
pub use license::{PackageLicense, LicenseReport, LicenseSummary, LicenseTypes};
pub use policy::{LicensePolicy, ViolationLevel, Violation, ViolationSummary};
