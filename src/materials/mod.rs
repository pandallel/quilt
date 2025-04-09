pub mod types;
// registry module removed as it needs significant refactoring
pub mod repository;

#[cfg(test)]
mod tests;

pub use types::{Material, MaterialFileType, MaterialStatus};
// Material registry will be reimplemented for actor model
pub use repository::{MaterialRepository, RepositoryError};
