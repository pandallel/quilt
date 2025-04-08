use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

use crate::materials::types::Material;

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
    /// Materials that were successfully found
    pub found: Vec<Material>,
    /// Materials that failed to process
    pub failed: Vec<Material>,
}

/// A scanner for discovering material files in directories
pub struct DirectoryScanner {
    /// Base directory to scan from
    base_dir: PathBuf,
    /// Whether to ignore hidden files and directories
    ignore_hidden: bool,
    /// Patterns to exclude from scanning
    exclude_patterns: Vec<String>,
}

impl DirectoryScanner {
    /// Create a new DirectoryScanner for the given base directory
    pub fn new<P: AsRef<Path>>(base_dir: P) -> ScanResult<Self> {
        let base_dir = base_dir.as_ref().to_path_buf();
        if !base_dir.exists() {
            return Err(ScanError::PathNotFound(base_dir));
        }
        Ok(Self {
            base_dir,
            ignore_hidden: true,
            exclude_patterns: Vec::new(),
        })
    }

    /// Set whether to ignore hidden files and directories
    pub fn ignore_hidden(mut self, ignore: bool) -> Self {
        self.ignore_hidden = ignore;
        self
    }

    /// Add patterns to exclude from scanning
    pub fn exclude<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.exclude_patterns
            .extend(patterns.into_iter().map(Into::into));
        self
    }

    /// Check if a path should be excluded based on exclude patterns
    fn should_exclude(&self, entry: &walkdir::DirEntry) -> bool {
        let path = entry.path().to_string_lossy();
        self.exclude_patterns
            .iter()
            .any(|pattern| path.contains(pattern))
    }

    /// Scan the base directory for material files
    pub fn scan(&self) -> ScanResult<ScanResults> {
        let mut results = ScanResults {
            found: Vec::new(),
            failed: Vec::new(),
        };

        // First collect all the files we want to process
        let mut files_to_process = Vec::new();

        let walker = WalkDir::new(&self.base_dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| {
                // Always allow the root directory
                if e.depth() == 0 {
                    return true;
                }

                // Check exclude patterns
                if self.should_exclude(e) {
                    return false;
                }

                // Check for hidden files/directories if enabled
                if self.ignore_hidden {
                    !e.file_name().to_str().map_or(false, |s| s.starts_with('.'))
                } else {
                    true
                }
            });

        // Collect all valid files first
        for entry in walker.filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }

            let relative_path = entry
                .path()
                .strip_prefix(&self.base_dir)
                .map(|p| p.to_string_lossy().into_owned())
                .unwrap_or_else(|_| entry.path().to_string_lossy().into_owned());

            files_to_process.push(relative_path);
        }

        // Now create Material instances for all the files
        for path in files_to_process {
            let material = Material::new(path);
            results.found.push(material);
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
        fs::create_dir_all(temp_dir.path().join("target/debug")).unwrap();

        File::create(temp_dir.path().join("docs/test1.md")).unwrap();
        File::create(temp_dir.path().join("docs/test2.md")).unwrap();
        File::create(temp_dir.path().join("notes/note.md")).unwrap();
        File::create(temp_dir.path().join("target/debug/output.txt")).unwrap();

        temp_dir
    }

    #[test]
    fn test_scan_finds_all_files() {
        let temp_dir = setup_test_dir();
        let scanner = DirectoryScanner::new(temp_dir.path()).unwrap();

        let results = scanner.scan().unwrap();

        assert_eq!(results.found.len(), 4, "Should find all 4 files");
        assert!(results.failed.is_empty(), "Should have no failed files");
    }

    #[test]
    fn test_exclude_patterns() {
        let temp_dir = setup_test_dir();
        let scanner = DirectoryScanner::new(temp_dir.path())
            .unwrap()
            .exclude(vec!["target/"]);

        let results = scanner.scan().unwrap();

        assert_eq!(
            results.found.len(),
            3,
            "Should only find non-excluded files"
        );
        assert!(
            results
                .found
                .iter()
                .all(|m| !m.file_path.contains("target/")),
            "Should not include files from excluded directory"
        );
    }

    #[test]
    fn test_multiple_exclude_patterns() {
        let temp_dir = setup_test_dir();
        let scanner = DirectoryScanner::new(temp_dir.path())
            .unwrap()
            .exclude(vec!["target/", "docs/"]);

        let results = scanner.scan().unwrap();

        assert_eq!(
            results.found.len(),
            1,
            "Should only find files not matching any exclude pattern"
        );
        assert!(
            results
                .found
                .iter()
                .all(|m| !m.file_path.contains("target/") && !m.file_path.contains("docs/")),
            "Should not include files from any excluded directory"
        );
    }

    #[test]
    fn test_scan_nonexistent_directory() {
        let scanner = DirectoryScanner::new("nonexistent");

        assert!(matches!(scanner, Err(ScanError::PathNotFound(_))));
    }

    #[test]
    fn test_hidden_files_included_when_configured() {
        let temp_dir = setup_test_dir();
        File::create(temp_dir.path().join(".hidden.txt")).unwrap();

        let scanner = DirectoryScanner::new(temp_dir.path())
            .unwrap()
            .ignore_hidden(false);

        let results = scanner.scan().unwrap();

        assert!(
            results
                .found
                .iter()
                .any(|m| m.file_path.contains(".hidden.txt")),
            "Hidden files should be included when ignore_hidden is false"
        );
    }

    #[test]
    fn test_hidden_directories_included_when_configured() {
        let temp_dir = setup_test_dir();

        // Create various directory structures
        fs::create_dir_all(temp_dir.path().join(".hidden_dir")).unwrap();
        fs::create_dir_all(temp_dir.path().join(".hidden_dir/visible_subdir")).unwrap();
        File::create(temp_dir.path().join(".hidden_dir/visible_subdir/file1.txt")).unwrap();

        let scanner = DirectoryScanner::new(temp_dir.path())
            .unwrap()
            .ignore_hidden(false);

        let results = scanner.scan().unwrap();

        assert!(
            results
                .found
                .iter()
                .any(|m| m.file_path.contains("file1.txt")),
            "Files in hidden directories should be included when ignore_hidden is false"
        );
    }

    #[test]
    fn test_relative_paths() {
        let temp_dir = setup_test_dir();
        let scanner = DirectoryScanner::new(temp_dir.path()).unwrap();

        let results = scanner.scan().unwrap();

        for material in results.found {
            assert!(
                !material.file_path.starts_with('/'),
                "Paths should be relative"
            );
            assert!(
                Path::new(&material.file_path).is_relative(),
                "Paths should be relative"
            );
        }
    }

    #[test]
    fn test_hidden_files_ignored() {
        let temp_dir = setup_test_dir();
        File::create(temp_dir.path().join(".hidden.txt")).unwrap();

        let scanner = DirectoryScanner::new(temp_dir.path())
            .unwrap()
            .ignore_hidden(true);
        let results = scanner.scan().unwrap();

        assert!(
            !results
                .found
                .iter()
                .any(|m| m.file_path.contains(".hidden.txt")),
            "Hidden files should be ignored"
        );
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

        let scanner = DirectoryScanner::new(temp_dir.path())
            .unwrap()
            .ignore_hidden(true);
        let results = scanner.scan().unwrap();

        // Files under a hidden directory should not be found, even in visible subdirectories
        assert!(
            !results
                .found
                .iter()
                .any(|m| m.file_path.contains("file1.txt")),
            "Should not find files under hidden directories"
        );

        // Files in hidden subdirectories should not be found
        assert!(
            !results
                .found
                .iter()
                .any(|m| m.file_path.contains("file2.txt")),
            "Should not find files in hidden subdirectories of hidden directories"
        );
        assert!(
            !results
                .found
                .iter()
                .any(|m| m.file_path.contains("file3.txt")),
            "Should not find files in hidden subdirectories of visible directories"
        );
    }
}
