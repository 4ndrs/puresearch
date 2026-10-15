use clap::{Parser, Subcommand};
use puresearch::{establish_connection, index_file};

#[derive(Parser, Debug)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    commands: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Index { path: String },
    Search { query: String },
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let connection = &mut establish_connection()?;

    match args.commands {
        Commands::Index { path } => {
            dbg!(&path);

            let mut hasher = blake3::Hasher::new();

            for entry in std::fs::read_dir(path)? {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        eprintln!("failed to read entry: {error}");

                        continue;
                    }
                };

                let path = match std::path::absolute(entry.path()) {
                    Ok(path) => path,
                    Err(error) => {
                        eprintln!("failed to convert path to absolute: {error}");

                        continue;
                    }
                };

                if let Some(extension) = path.extension()
                    && (extension == "png" || extension == "jpg")
                {
                    hasher.reset();

                    if let Err(error) = hasher.update_mmap(&path) {
                        eprintln!("failed to update_mmap the file {}: {error}", path.display());

                        continue;
                    }

                    let hash = hasher.finalize();

                    let path = match path.to_str() {
                        Some(path) => path,
                        None => {
                            eprintln!("unable to extract a valid path for: {}", path.display());
                            continue;
                        }
                    };

                    let results = match index_file(connection, &hash, path) {
                        Ok(results) => results,
                        Err(error) => {
                            eprintln!("error while indexing the file in {path}: {error}");
                            continue;
                        }
                    };

                    if results.added_image {
                        println!("added new image with path: {path}")
                    } else if results.added_file {
                        println!("added new path to image id {}: {path}", results.image_id);
                    } else {
                        println!("image and file path already exists, skipping: {path}")
                    }
                }
            }
        }
        Commands::Search { query } => {
            dbg!(query);
        }
    }

    Ok(())
}
