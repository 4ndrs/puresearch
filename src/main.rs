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
                let path = std::path::absolute(entry?.path())?;

                if let Some(extension) = path.extension()
                    && (extension == "png" || extension == "jpg")
                {
                    hasher.update_mmap(&path)?;

                    let hash = hasher.finalize();

                    dbg!(hash);

                    hasher.reset();

                    let path = path.to_str().ok_or("path not a valid utf8")?;
                    let results = index_file(connection, &hash, path)?;

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
