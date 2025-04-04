pub mod events;
pub mod materials;

pub use materials::{Material, MaterialStatus, MaterialFileType, MaterialRegistry};
pub use materials::scanner::DirectoryScanner;

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
