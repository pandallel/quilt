#[cfg(test)]
mod scanner_tests {
    use std::path::PathBuf;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::TempDir;
    use crate::materials::scanner::DirectoryScanner;

    /// Creates a temporary directory with test files for scanning
    fn setup_test_directory() -> (TempDir, PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let temp_path = temp_dir.path().to_path_buf();

        // Create a markdown file
        let md_path = temp_path.join("test_doc.md");
        let mut md_file = File::create(&md_path).expect("Failed to create markdown file");
        writeln!(md_file, "# Test Document\n\nThis is a test document.")
            .expect("Failed to write to file");

        // Create a subdirectory with another file
        let subdir_path = temp_path.join("subdir");
        fs::create_dir(&subdir_path).expect("Failed to create subdirectory");

        let subdir_file_path = subdir_path.join("nested_doc.md");
        let mut subdir_file =
            File::create(&subdir_file_path).expect("Failed to create nested file");
        writeln!(
            subdir_file,
            "# Nested Document\n\nThis is a nested document."
        )
        .expect("Failed to write to file");

        (temp_dir, temp_path)
    }

    #[test]
    fn test_scanner_basic_scan() {
        let (temp_dir, temp_path) = setup_test_directory();

        // Create scanner
        let scanner = DirectoryScanner::new(&temp_path).expect("Failed to create scanner");

        // Perform the scan
        let results = scanner.scan().expect("Failed to scan directory");

        // We should find 2 markdown files
        assert_eq!(results.found.len(), 2, "Expected 2 found files");
        assert!(results.failed.is_empty(), "Expected no failed files");

        // Verify file paths
        let found_paths: Vec<&str> = results.found.iter().map(|m| m.file_path.as_str()).collect();

        assert!(
            found_paths.contains(&"test_doc.md"),
            "Should find root file"
        );
        assert!(
            found_paths.contains(&"subdir/nested_doc.md"),
            "Should find nested file"
        );

        // Keep temp directory around until end of test
        drop(temp_dir);
    }

    #[test]
    fn test_scanner_with_exclude_patterns() {
        let (temp_dir, temp_path) = setup_test_directory();

        // Create scanner that excludes the subdir
        let scanner = DirectoryScanner::new(&temp_path)
            .expect("Failed to create scanner")
            .exclude(vec!["subdir"]);

        // Perform the scan
        let results = scanner.scan().expect("Failed to scan directory");

        // We should find only 1 file (the one in the root directory)
        assert_eq!(results.found.len(), 1, "Expected 1 found file");
        assert!(results.failed.is_empty(), "Expected no failed files");

        // Verify only the root file is found
        assert_eq!(
            results.found[0].file_path, "test_doc.md",
            "Should only find root file"
        );

        // Keep temp directory around until end of test
        drop(temp_dir);
    }
}
