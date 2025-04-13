// Quilt - Material Swatching Module
//
// This module handles the swatching of materials, transforming cut chunks
// into semantic embeddings (swatches).

mod actor;

pub use actor::messages::{OperationComplete, SwatchingError};
pub use actor::SwatchingActor;

#[cfg(test)]
mod tests {
    // Integration tests for the swatching module
}
