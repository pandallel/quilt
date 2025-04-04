use std::env;
use std::process;

use quilt::{MaterialRegistry, DirectoryScanner};

fn print_usage(program: &str) {
    eprintln!("Usage: {} <directory> [--exclude path1,path2,...]", program);
    eprintln!("\nOptions:");
    eprintln!("  --exclude    Comma-separated list of paths to exclude");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    let mut path = None;
    let mut exclude_paths = Vec::new();
    let mut i = 1;
    
    while i < args.len() {
        match args[i].as_str() {
            "--exclude" => {
                if i + 1 >= args.len() {
                    eprintln!("Error: --exclude requires a comma-separated list of paths");
                    print_usage(&args[0]);
                    process::exit(1);
                }
                exclude_paths = args[i + 1].split(',').map(String::from).collect();
                i += 2;
            }
            arg => {
                if path.is_none() {
                    path = Some(arg.to_string());
                } else {
                    eprintln!("Error: Unexpected argument '{}'", arg);
                    print_usage(&args[0]);
                    process::exit(1);
                }
                i += 1;
            }
        }
    }

    let path = path.expect("Directory path is required");
    let mut registry = MaterialRegistry::new();
    
    match DirectoryScanner::new(&path, &mut registry) {
        Ok(scanner) => {
            // Apply exclude paths if any were provided
            let mut scanner = if !exclude_paths.is_empty() {
                scanner.exclude(exclude_paths)
            } else {
                scanner
            };

            match scanner.scan() {
                Ok(results) => {
                    println!("\nScan Results:");
                    println!("-------------");
                    
                    // Get all materials from the registry
                    let registered_materials = registry.list_all();
                    println!("Materials in registry: {}", registered_materials.len());
                    for material in registered_materials {
                        println!("  - {} ({})", material.file_path, material.status);
                        if let Some(error) = material.error {
                            println!("    Error: {}", error);
                        }
                    }
                    
                    // Show failed registrations from scan results
                    if !results.failed.is_empty() {
                        println!("\nFailed to register {} materials:", results.failed.len());
                        for material in results.failed {
                            println!("  - {}", material.file_path);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error scanning directory: {}", err);
                    process::exit(1);
                }
            }
        }
        Err(err) => {
            eprintln!("Error creating scanner: {}", err);
            process::exit(1);
        }
    }
} 