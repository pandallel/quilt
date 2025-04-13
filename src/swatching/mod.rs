// Quilt - Material Swatching Module
//
// This module handles the swatching of materials, transforming cut chunks
// into semantic embeddings (swatches).

mod actor;
mod repository;
mod swatch;

pub use actor::messages::{OperationComplete, SwatchingError};
pub use actor::SwatchingActor;
pub use repository::{Result, SwatchRepository, SwatchRepositoryError};
pub use swatch::Swatch;

#[cfg(test)]
mod tests {
    // Integration tests for the swatching module
}
