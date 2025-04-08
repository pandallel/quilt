pub mod types;
// registry module removed as it needs significant refactoring
pub mod scanner;

pub use types::{Material, MaterialStatus, MaterialFileType};
// Material registry will be reimplemented for actor model
pub use scanner::{DirectoryScanner, ScanError, ScanResult, ScanResults}; 