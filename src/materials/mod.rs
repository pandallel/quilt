pub mod types;
// registry module removed as it needs significant refactoring
pub mod scanner;
pub mod repository;

pub use types::{Material, MaterialStatus, MaterialFileType};
// Material registry will be reimplemented for actor model
pub use scanner::{DirectoryScanner, ScanError, ScanResult, ScanResults};
pub use repository::{MaterialRepository, RepositoryError}; 