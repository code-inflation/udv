use clap::{Parser, Subcommand};
use std::fs::{self, File};
use std::io::Write;
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
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init => init(),
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

    let mut gitignore_file = match File::create(&gitignore_path) {
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

    if let Err(e) = File::create(&config_path) {
        eprintln!("Failed to create config file: {}", e);
        process::exit(1);
    }

    println!("udv project initialized successfully.");
}
