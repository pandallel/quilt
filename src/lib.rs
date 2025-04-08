// Quilt - A local-first, modular memory and context engine
//
// This codebase is being refactored to implement an actor-based architecture.
// Current status: Basic structure with Material types and Directory Scanner.
// Next steps: Implement the actor framework and Material Repository.

pub mod materials;

// Keep only the material types and scanner exports
pub use materials::{Material, MaterialStatus, MaterialFileType};
pub use materials::scanner::DirectoryScanner;

// This function is only here for testing and can be removed later
pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
