use std::path::PathBuf;

use archiver::types::{CreateOptions, OpenOptions};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "archiver", version = "0.1.0", about = "Universal archive tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List contents of an archive
    List {
        path: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Extract all files from an archive
    Extract {
        path: PathBuf,
        #[arg(short, long, default_value = ".")]
        dest: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Extract a single file from an archive
    ExtractFile {
        path: PathBuf,
        entry: String,
        #[arg(short, long, default_value = ".")]
        dest: PathBuf,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Preview a file from an archive (output to stdout)
    Preview {
        path: PathBuf,
        entry: String,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Create a new archive
    Create {
        output: PathBuf,
        #[arg(required = true)]
        sources: Vec<PathBuf>,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Add files to an existing archive
    Add {
        path: PathBuf,
        #[arg(required = true)]
        sources: Vec<PathBuf>,
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Split an archive into parts
    Split {
        path: PathBuf,
        #[arg(short, long)]
        size: u64,
        #[arg(short, long, default_value = ".")]
        output_dir: PathBuf,
    },
    /// Merge split parts back into a single file
    Merge {
        output: PathBuf,
        #[arg(required = true)]
        parts: Vec<PathBuf>,
    },
    /// Detect archive format
    Detect {
        path: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::List { path, password } => {
            let opts = password.map(|p| OpenOptions {
                password: Some(p),
            });
            match archiver::list(&path, opts.as_ref()) {
                Ok(entries) => {
                    println!(
                        "{:<60} {:>12} {:>12} {:>5} {:>5}",
                        "Path", "Size", "Compressed", "Dir", "Enc"
                    );
                    println!("{}", "-".repeat(96));
                    for e in &entries {
                        println!(
                            "{:<60} {:>12} {:>12} {:>5} {:>5}",
                            e.path,
                            e.size,
                            e.compressed_size
                                .map(|s| s.to_string())
                                .unwrap_or_else(|| "-".to_string()),
                            if e.is_dir { "yes" } else { "no" },
                            if e.encrypted { "yes" } else { "no" },
                        );
                    }
                    println!("\nTotal: {} entries", entries.len());
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        Commands::Extract {
            path,
            dest,
            password,
        } => {
            let opts = password.map(|p| OpenOptions {
                password: Some(p),
            });
            archiver::extract_all(&path, &dest, opts.as_ref())
                .map(|_| println!("✓ Extracted to {}", dest.display()))
        }
        Commands::ExtractFile {
            path,
            entry,
            dest,
            password,
        } => {
            let opts = password.map(|p| OpenOptions {
                password: Some(p),
            });
            archiver::extract_file(&path, &entry, &dest, opts.as_ref())
                .map(|_| println!("✓ Extracted '{}' to {}", entry, dest.display()))
        }
        Commands::Preview {
            path,
            entry,
            password,
        } => {
            let opts = password.map(|p| OpenOptions {
                password: Some(p),
            });
            match archiver::preview(&path, &entry, opts.as_ref()) {
                Ok(data) => {
                    match String::from_utf8(data.clone()) {
                        Ok(text) => print!("{}", text),
                        Err(_) => {
                            println!("[Binary data, {} bytes]", data.len());
                            for (i, byte) in data.iter().take(256).enumerate() {
                                if i % 16 == 0 && i > 0 {
                                    println!();
                                }
                                print!("{:02x} ", byte);
                            }
                            println!();
                        }
                    }
                    Ok(())
                }
                Err(e) => Err(e),
            }
        }
        Commands::Create {
            output,
            sources,
            password,
        } => {
            let opts = password.map(|p| CreateOptions {
                password: Some(p),
                ..Default::default()
            });
            archiver::create(&output, &sources, opts.as_ref())
                .map(|_| println!("✓ Created {}", output.display()))
        }
        Commands::Add {
            path,
            sources,
            password,
        } => {
            let opts = password.map(|p| CreateOptions {
                password: Some(p),
                ..Default::default()
            });
            archiver::add(&path, &sources, opts.as_ref())
                .map(|_| println!("✓ Added files to {}", path.display()))
        }
        Commands::Split {
            path,
            size,
            output_dir,
        } => match archiver::split_archive(&path, size, &output_dir) {
            Ok(parts) => {
                println!("✓ Split into {} parts:", parts.len());
                for p in &parts {
                    println!("  {}", p.display());
                }
                Ok(())
            }
            Err(e) => Err(e),
        },
        Commands::Merge { output, parts } => archiver::merge_parts(&parts, &output)
            .map(|_| println!("✓ Merged into {}", output.display())),
        Commands::Detect { path } => match archiver::detect(&path) {
            Ok(format) => {
                println!("Detected format: {}", format);
                Ok(())
            }
            Err(e) => Err(e),
        },
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
