use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

use crate::materials::types::Material;
use crate::materials::registry::MaterialRegistry;

/// Errors that can occur during directory scanning
#[derive(Error, Debug)]
pub enum ScanError {
    #[error("Path does not exist: {0}")]
    PathNotFound(PathBuf),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for directory scanning operations
pub type ScanResult<T> = Result<T, ScanError>;

/// Results of a directory scan
#[derive(Debug)]
pub struct ScanResults {
    /// Materials that were successfully registered
    pub registered: Vec<Material>,
    /// Materials that failed to register (e.g., duplicates)
    pub failed: Vec<Material>,
}

/// A scanner for discovering material files in directories
pub struct DirectoryScanner<'a> {
    /// Base directory to scan from
    base_dir: PathBuf,
    /// Registry to register materials in
    registry: &'a mut MaterialRegistry,
    /// Whether to ignore hidden files and directories
    ignore_hidden: bool,
}

impl<'a> DirectoryScanner<'a> {
    /// Create a new DirectoryScanner for the given base directory and registry
    pub fn new<P: AsRef<Path>>(base_dir: P, registry: &'a mut MaterialRegistry) -> ScanResult<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            return Err(ScanError::PathNotFound(base_dir));
        }
        Ok(Self { 
            base_dir, 
            registry,
            ignore_hidden: true,  // Default to ignoring hidden files
        })
    }

    /// Set whether to ignore hidden files and directories
    pub fn ignore_hidden(mut self, ignore: bool) -> Self {
        self.ignore_hidden = ignore;
        self
    }

    /// Scan the base directory for material files and register them
    pub fn scan(&mut self) -> ScanResult<ScanResults> {
        let mut results = ScanResults {
            registered: Vec::new(),
            failed: Vec::new(),
        };

        let walker = WalkDir::new(&self.base_dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| {
                // Always allow the root directory
                if e.depth() == 0 {
                    return true;
                }
                
                // If we're not ignoring hidden files, allow everything
                if !self.ignore_hidden {
                    return true;
                }
                
                // Only check if the current entry's name starts with a dot
                !e.file_name()
                    .to_str()
                    .map_or(false, |s| s.starts_with('.'))
            });

        for entry in walker.filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }

            let relative_path = entry.path()
                .strip_prefix(&self.base_dir)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| entry.path().to_string_lossy().into_owned());

            let material = Material::new(relative_path);

            match self.registry.upsert(material.clone()) {
                Some(registered) => results.registered.push(registered),
                None => results.failed.push(material),
            }
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    fn setup_test_dir() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        
        // Create some test files
        fs::create_dir_all(temp_dir.path().join("docs")).unwrap();
        fs::create_dir_all(temp_dir.path().join("notes")).unwrap();
        
        File::create(temp_dir.path().join("docs/test1.md")).unwrap();
        File::create(temp_dir.path().join("docs/test2.md")).unwrap();
        File::create(temp_dir.path().join("notes/note.md")).unwrap();
        File::create(temp_dir.path().join("not-markdown.txt")).unwrap();
        
        temp_dir
    }

    #[test]
    fn test_scan_finds_and_registers_all_files() {
        let temp_dir = setup_test_dir();
        let mut registry = MaterialRegistry::new();
        let mut scanner = DirectoryScanner::new(temp_dir.path(), &mut registry).unwrap();
        
        let results = scanner.scan().unwrap();
        
        assert_eq!(results.registered.len(), 4, "Should register all 4 files");
        assert!(results.failed.is_empty(), "Should have no failed registrations");
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let mut registry = MaterialRegistry::new();
        let scanner = DirectoryScanner::new("nonexistent", &mut registry);
        
        assert!(matches!(scanner, Err(ScanError::PathNotFound(_))));
    }

    #[test]
    fn test_hidden_files_included_when_configured() {
        let temp_dir = setup_test_dir();
        File::create(temp_dir.path().join(".hidden.txt")).unwrap();
        
        let mut registry = MaterialRegistry::new();
        let mut scanner = DirectoryScanner::new(temp_dir.path(), &mut registry)
            .unwrap()
            .ignore_hidden(false);

        let results = scanner.scan().unwrap();
        
        assert!(results.registered.iter().any(|m| m.file_path.contains(".hidden.txt")),
            "Hidden files should be included when ignore_hidden is false");
    }

    #[test]
    fn test_hidden_directories_included_when_configured() {
        let temp_dir = setup_test_dir();
        
        // Create various directory structures
        fs::create_dir_all(temp_dir.path().join(".hidden_dir")).unwrap();
        fs::create_dir_all(temp_dir.path().join(".hidden_dir/visible_subdir")).unwrap();
        File::create(temp_dir.path().join(".hidden_dir/visible_subdir/file1.txt")).unwrap();
        
        let mut registry = MaterialRegistry::new();
        let mut scanner = DirectoryScanner::new(temp_dir.path(), &mut registry)
            .unwrap()
            .ignore_hidden(false);

        let results = scanner.scan().unwrap();
        
        assert!(results.registered.iter().any(|m| m.file_path.contains("file1.txt")),
            "Files in hidden directories should be included when ignore_hidden is false");
    }

    #[test]
    fn test_relative_paths() {
        let temp_dir = setup_test_dir();
        let mut registry = MaterialRegistry::new();
        let mut scanner = DirectoryScanner::new(temp_dir.path(), &mut registry).unwrap();
        
        let results = scanner.scan().unwrap();
        
        for material in results.registered {
            assert!(!material.file_path.starts_with('/'),
                "Paths should be relative");
            assert!(Path::new(&material.file_path).is_relative(),
                "Paths should be relative");
        }
    }

    #[test]
    fn test_duplicate_registrations() {
        let temp_dir = setup_test_dir();
        let mut registry = MaterialRegistry::new();
        
        // First scan
        let mut scanner1 = DirectoryScanner::new(temp_dir.path(), &mut registry).unwrap();
        let results1 = scanner1.scan().unwrap();
        assert_eq!(results1.registered.len(), 4, "Should register all 4 files initially");
        assert!(results1.failed.is_empty(), "Should have no failures initially");

        // Second scan
        let mut scanner2 = DirectoryScanner::new(temp_dir.path(), &mut registry).unwrap();
        let results2 = scanner2.scan().unwrap();
        assert!(results2.registered.is_empty(), "Should not register any new files");
        assert_eq!(results2.failed.len(), 4, "All files should fail as duplicates");
    }

    #[test]
    fn test_hidden_files_ignored() {
        let temp_dir = setup_test_dir();
        File::create(temp_dir.path().join(".hidden.txt")).unwrap();
        
        let mut registry = MaterialRegistry::new();
        let mut scanner = DirectoryScanner::new(temp_dir.path(), &mut registry).unwrap();
        let results = scanner.scan().unwrap();
        
        assert!(!results.registered.iter().any(|m| m.file_path.contains(".hidden.txt")),
            "Hidden files should be ignored");
    }

    #[test]
    fn test_hidden_directories_are_skipped() {
        let temp_dir = setup_test_dir();
        
        // Create various directory structures
        fs::create_dir_all(temp_dir.path().join(".hidden_dir")).unwrap();
        fs::create_dir_all(temp_dir.path().join(".hidden_dir/visible_subdir")).unwrap();
        fs::create_dir_all(temp_dir.path().join(".hidden_dir/.hidden_subdir")).unwrap();
        fs::create_dir_all(temp_dir.path().join("visible_dir/.hidden_subdir")).unwrap();
        
        // Create some test files
        File::create(temp_dir.path().join(".hidden_dir/visible_subdir/file1.txt")).unwrap();
        File::create(temp_dir.path().join(".hidden_dir/.hidden_subdir/file2.txt")).unwrap();
        File::create(temp_dir.path().join("visible_dir/.hidden_subdir/file3.txt")).unwrap();
        
        let mut registry = MaterialRegistry::new();
        let mut scanner = DirectoryScanner::new(temp_dir.path(), &mut registry).unwrap();
        let results = scanner.scan().unwrap();
        
        // Files under a hidden directory should not be found, even in visible subdirectories
        assert!(!results.registered.iter().any(|m| m.file_path.contains("file1.txt")),
            "Should not find files under hidden directories");
            
        // Files in hidden subdirectories should not be found
        assert!(!results.registered.iter().any(|m| m.file_path.contains("file2.txt")),
            "Should not find files in hidden subdirectories of hidden directories");
        assert!(!results.registered.iter().any(|m| m.file_path.contains("file3.txt")),
            "Should not find files in hidden subdirectories of visible directories");
    }
} 