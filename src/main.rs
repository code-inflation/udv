use clap::{Parser, Subcommand};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;

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
    Add { path: String },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init()?,
        Commands::Add { path } => add(Path::new(&path))?,
    }

    Ok(())
}

fn init() -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new("./.git").exists() {
        return Err("Current directory is not a Git repository.".into());
    }
    if Path::new("./.dvc").exists() {
        return Err(
            "Current directory already contains data versioning config (.dvc folder).".into(),
        );
    }
    if Path::new("./.udv").exists() {
        return Err(
            "Current directory already contains data versioning config (.udv folder).".into(),
        );
    }

    println!("Initializing udv project...");

    let udv_dir = "./.udv";
    let gitignore_path = format!("{}/.gitignore", udv_dir);
    let config_path = format!("{}/config", udv_dir);

    println!("Initializing udv project in the current Git repository...");

    fs::create_dir(udv_dir)?;

    let mut gitignore_file = File::create(gitignore_path)?;

    let gitignore_contents = "/config.local\n/tmp\n/cache";
    gitignore_file.write_all(gitignore_contents.as_bytes())?;

    File::create(config_path)?;

    println!("udv project initialized successfully.");
    Ok(())
}

fn hash_file(path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

pub fn add(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !path.exists() {
        return Err(format!("Error: The specified path does not exist: {:?}", path).into());
    }

    // Create .udv/cache directory if it doesn't exist
    let cache_dir = Path::new(".udv/cache");
    fs::create_dir_all(cache_dir)?;

    if path.is_file() {
        add_file(path)?;
    } else if path.is_dir() {
        add_directory(path)?;
    }

    Ok(())
}

fn add_file(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let file_hash = hash_file(path)?;
    let cache_path = Path::new(".udv/cache")
        .join(&file_hash[..2])
        .join(&file_hash[2..]);

    // Create cache subdirectories if they don't exist
    fs::create_dir_all(cache_path.parent().unwrap())?;

    // Copy the file to cache (use copy instead of move to keep the original file)
    fs::copy(path, &cache_path)?;

    // Create .dvc file
    let dvc_path = path.with_file_name(format!(
        "{}.dvc",
        path.file_name().unwrap().to_str().unwrap()
    ));
    let dvc_content = json!({
        "outs": [{
            "md5": file_hash,
            "path": path.to_str().unwrap()
        }]
    });

    let mut dvc_file = File::create(dvc_path)?;
    dvc_file.write_all(serde_json::to_string_pretty(&dvc_content)?.as_bytes())?;

    // Update .gitignore for this file
    update_gitignore(path)?;

    println!("Added {:?}", path);
    Ok(())
}

fn add_directory(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        if entry_path.is_file() {
            add_file(&entry_path)?;
        } else if entry_path.is_dir() {
            add_directory(&entry_path)?;
        }
    }

    println!("Added directory {:?}", path);
    Ok(())
}

fn update_gitignore(path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let gitignore_path = Path::new(".gitignore");
    let mut gitignore_content = String::new();

    if gitignore_path.exists() {
        let mut file = File::open(gitignore_path)?;
        file.read_to_string(&mut gitignore_content)?;
    }

    let path_str = path.to_str().unwrap();
    if !gitignore_content.contains(path_str) {
        gitignore_content.push_str(&format!("\n{}", path_str));
        let mut file = File::create(gitignore_path)?;
        file.write_all(gitignore_content.as_bytes())?;
    }

    Ok(())
}
