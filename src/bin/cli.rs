use clap::{Parser, Subcommand};
use puresearch::{
    clip::{generate_image_embedding, get_clip_model, get_device_and_config, get_scores},
    establish_connection, get_all_embeddings, get_paths, get_tokenizer, index_file,
    is_missing_embedding, save_embedding,
};

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

    let (device, config) = get_device_and_config();

    let model = get_clip_model(&config, &device)?;

    match args.commands {
        Commands::Index { path } => {
            dbg!(&path);

            let mut hasher = blake3::Hasher::new();
            let mut failed_files = 0;

            for entry in std::fs::read_dir(path)? {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        eprintln!("failed to read entry: {error}");

                        failed_files += 1;

                        continue;
                    }
                };

                let path = match std::path::absolute(entry.path()) {
                    Ok(path) => path,
                    Err(error) => {
                        eprintln!("failed to convert path to absolute: {error}");

                        failed_files += 1;

                        continue;
                    }
                };

                if let Some(extension) = path.extension()
                    && (extension == "png" || extension == "jpg")
                {
                    hasher.reset();

                    if let Err(error) = hasher.update_mmap(&path) {
                        eprintln!("failed to update_mmap the file {}: {error}", path.display());

                        failed_files += 1;

                        continue;
                    }

                    let hash = hasher.finalize();

                    let path = match path.to_str() {
                        Some(path) => path,
                        None => {
                            eprintln!("unable to extract a valid path for: {}", path.display());

                            failed_files += 1;

                            continue;
                        }
                    };

                    let results = index_file(connection, &hash, path)?;

                    if is_missing_embedding(connection, results.image_id)? {
                        match generate_image_embedding(path, &config, &device, &model) {
                            Ok(embedding) => {
                                save_embedding(connection, results.image_id, &embedding)?
                            }
                            Err(error) => eprintln!(
                                "error while generating the embedding for image id {}: {error}",
                                results.image_id
                            ),
                        }
                    }

                    if results.added_image {
                        println!("added new image with path: {path}")
                    } else if results.added_file {
                        println!("added new path to image id {}: {path}", results.image_id);
                    } else {
                        println!("image and file path already exists, skipping: {path}")
                    }
                }
            }

            if failed_files > 0 {
                return Err(format!("{failed_files} file(s) failed to index").into());
            }
        }
        Commands::Search { query } => {
            dbg!(&query);

            let tokenizer = get_tokenizer()?;

            let (image_ids, embeddings): (Vec<_>, Vec<_>) =
                get_all_embeddings(connection)?.into_iter().unzip();

            let scores = get_scores(&model, tokenizer, &device, &embeddings, &query)?;

            let mut scored_image_ids: Vec<(_, _)> = image_ids.iter().zip(scores).collect();

            scored_image_ids.sort_by(|(_, score_a), (_, score_b)| score_b.total_cmp(score_a));

            let paths_map = get_paths(connection, &image_ids)?;

            for (image_id, score) in scored_image_ids {
                let image_paths = paths_map
                    .get(image_id)
                    .map(|paths| paths.join(", "))
                    .unwrap_or_else(|| "no paths".to_string());

                println!("{score} {image_paths}");
            }
        }
    }

    Ok(())
}
