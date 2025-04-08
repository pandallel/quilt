// Discovery module for Quilt
// Responsible for discovering materials in directories

pub mod actor;
pub mod scanner;

#[cfg(test)]
mod tests;

// Re-export the actor for easy access
pub use self::actor::DiscoveryActor;
// Re-export the scanner for easy access
pub use self::scanner::{DirectoryScanner, ScanError, ScanResult, ScanResults};
