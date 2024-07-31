use clap::{Parser, Subcommand};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process;

/// Blazingly fast data versioning
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes a new udv project in the current directory.
    Init,
    /// TODO implement dvc add functionality
    /// TODO --compress flag that archives small files together to reduce S3 operations etc.
    Add,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init(),
        Commands::Add => add(),
    }
}

fn init() {
    if !Path::new("./.git").exists() {
        eprintln!("Error: Current directory is not a Git repository.");
        process::exit(1);
    }
    if Path::new("./.dvc").exists() {
        eprintln!(
            "Error: Current directory already contains data versioning config (.dvc folder)."
        );
        process::exit(1);
    }
    if Path::new("./.udv").exists() {
        eprintln!(
            "Error: Current directory already contains data versioning config (.udv folder)."
        );
        process::exit(1);
    }
    println!("Initializing udv project...");
    // Implementation for creating the .udv directory and initial config files goes here.
    let udv_dir = "./.udv";
    let gitignore_path = format!("{}/.gitignore", udv_dir);
    let config_path = format!("{}/config", udv_dir);

    println!("Initializing udv project in the current Git repository...");

    if let Err(e) = fs::create_dir(udv_dir) {
        eprintln!("Failed to create .udv directory: {}", e);
        process::exit(1);
    }

    let mut gitignore_file = match File::create(gitignore_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to create .gitignore file: {}", e);
            process::exit(1);
        }
    };

    let gitignore_contents = "/config.local\n/tmp\n/cache";
    if let Err(e) = gitignore_file.write_all(gitignore_contents.as_bytes()) {
        eprintln!("Failed to write to .gitignore: {}", e);
        process::exit(1);
    }

    if let Err(e) = File::create(config_path) {
        eprintln!("Failed to create config file: {}", e);
        process::exit(1);
    }

    println!("udv project initialized successfully.");
}

fn add() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Error: Please specify a file or directory to add.");
        process::exit(1);
    }

    let path = Path::new(&args[2]);
    if !path.exists() {
        eprintln!("Error: The specified path does not exist.");
        process::exit(1);
    }

    // Create .udv/cache directory if it doesn't exist
    let cache_dir = Path::new(".udv/cache");
    fs::create_dir_all(cache_dir).expect("Failed to create .udv/cache directory");

    if path.is_file() {
        add_file(path);
    } else if path.is_dir() {
        add_directory(path);
    }

    update_gitignore(path);
}

fn add_file(path: &Path) {
    let file_hash = hash_file(path);
    let cache_path = Path::new(".udv/cache")
        .join(&file_hash[..2])
        .join(&file_hash[2..]);

    // Create cache subdirectories if they don't exist
    fs::create_dir_all(cache_path.parent().unwrap())
        .expect("Failed to create cache subdirectories");

    // Move the file to cache
    fs::copy(path, &cache_path).expect("Failed to copy file to cache");

    // Create .dvc file
    let dvc_path = path.with_extension("dvc");
    let dvc_content = json!({
        "outs": [{
            "md5": file_hash,
            "path": path.to_str().unwrap()
        }]
    });

    let mut dvc_file = File::create(dvc_path).expect("Failed to create .dvc file");
    dvc_file
        .write_all(
            serde_json::to_string_pretty(&dvc_content)
                .unwrap()
                .as_bytes(),
        )
        .expect("Failed to write .dvc file");

    println!("Added {:?}", path);
}

fn add_directory(path: &Path) {
    for entry in fs::read_dir(path).expect("Failed to read directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();
        if path.is_file() {
            add_file(&path);
        } else if path.is_dir() {
            add_directory(&path);
        }
    }
}

fn hash_file(path: &Path) -> String {
    let mut file = File::open(path).expect("Failed to open file");
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer).expect("Failed to read file");
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    format!("{:x}", hasher.finalize())
}

fn update_gitignore(path: &Path) {
    let gitignore_path = Path::new(".gitignore");
    let mut gitignore_content = String::new();

    if gitignore_path.exists() {
        let mut file = File::open(gitignore_path).expect("Failed to open .gitignore");
        file.read_to_string(&mut gitignore_content)
            .expect("Failed to read .gitignore");
    }

    let path_str = path.to_str().unwrap();
    if !gitignore_content.contains(path_str) {
        gitignore_content.push_str(&format!("\n{}", path_str));
        let mut file = File::create(gitignore_path).expect("Failed to create .gitignore");
        file.write_all(gitignore_content.as_bytes())
            .expect("Failed to write .gitignore");
    }
}
