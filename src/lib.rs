mod schema;

pub mod clip;

use std::collections::HashMap;

use candle_core::{Device, Tensor};
use diesel::{insert_into, update};
use dotenvy::dotenv;
use tokenizers::Tokenizer;

use diesel::prelude::*;

pub fn establish_connection() -> Result<SqliteConnection, Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")?;

    Ok(SqliteConnection::establish(&database_url)?)
}

pub struct IndexResults {
    pub image_id: i32,
    pub added_file: bool,
    pub added_image: bool,
}

pub fn index_file(
    connection: &mut SqliteConnection,
    hash: &blake3::Hash,
    path: &str,
) -> QueryResult<IndexResults> {
    use schema::{files, images};

    connection.transaction(|connection| {
        let inserted_image_row = insert_into(images::table)
            .values(images::blake3.eq(hash.as_bytes()))
            .on_conflict(images::blake3)
            .do_nothing()
            .execute(connection)?;

        let image_id: i32 = images::table
            .filter(images::blake3.eq(hash.as_bytes()))
            .select(images::id)
            .first(connection)?;

        let inserted_file_row;

        {
            use diesel::query_dsl::methods::FilterDsl;

            inserted_file_row = insert_into(files::table)
                .values((files::path.eq(path), files::image_id.eq(image_id)))
                .on_conflict(files::path)
                .do_update()
                .set(files::image_id.eq(image_id))
                .filter(files::image_id.ne(image_id))
                .execute(connection)?;
        }

        Ok(IndexResults {
            image_id,
            // this added file can also be an update
            // eg. the image's content change but the path is the same
            // and already in the database FIXME
            added_file: inserted_file_row != 0,
            added_image: inserted_image_row != 0,
        })
    })
}

pub fn is_missing_embedding(connection: &mut SqliteConnection, image_id: i32) -> QueryResult<bool> {
    use schema::images;

    // im trying to do something like this
    // select id from images where id = {image_id} and embedding is null;
    // just to check if the id has missing embedding
    let result: Option<i32> = images::table
        .select(images::id)
        .filter(images::id.eq(image_id))
        .filter(images::embedding.is_null())
        .first(connection)
        .optional()?;

    match result {
        Some(_) => return Ok(true),
        None => return Ok(false),
    }
}

pub fn save_embedding(
    connection: &mut SqliteConnection,
    image_id: i32,
    embedding: &[u8],
) -> QueryResult<()> {
    use schema::images;

    update(images::table)
        .filter(images::id.eq(image_id))
        .set(images::embedding.eq(embedding))
        .execute(connection)?;

    Ok(())
}

pub fn get_all_embeddings(connection: &mut SqliteConnection) -> QueryResult<Vec<(i32, Vec<u8>)>> {
    use schema::images;

    // select id,embedding from images where embedding is not null;
    let results = images::table
        .select((images::id, images::embedding.assume_not_null()))
        .filter(images::embedding.is_not_null())
        .load::<(i32, Vec<u8>)>(connection)?;

    Ok(results)
}

pub fn get_paths(
    connection: &mut SqliteConnection,
    image_ids: &Vec<i32>,
) -> Result<HashMap<i32, Vec<String>>, Box<dyn std::error::Error>> {
    use schema::files;

    // images can have multiple paths so i will need to have all of them
    // select image_id,path from files where image_id in {image_ids};
    let results = files::table
        .select((files::image_id, files::path))
        .filter(files::image_id.eq_any(image_ids))
        .load::<(i32, String)>(connection)?;

    let mut hashmap: HashMap<i32, Vec<String>> = HashMap::new();

    // TODO check entry api
    for (image_id, path) in results {
        match hashmap.get_mut(&image_id) {
            Some(paths) => paths.push(path),
            None => {
                hashmap.insert(image_id, Vec::from([path]));
            }
        }
    }

    Ok(hashmap)
}

pub fn get_tokenizer() -> Result<Tokenizer, Box<dyn std::error::Error>> {
    let tokenizer = Tokenizer::from_file("tokenizer.json").map_err(|error| format!("{error}"))?;

    Ok(tokenizer)
}

fn generate_tokens(
    text: &str,
    tokenizer: &Tokenizer,
    device: &Device,
) -> Result<Tensor, Box<dyn std::error::Error>> {
    let tokens = tokenizer
        .encode(text, true)
        .map_err(|error| format!("{error}"))?;

    let tokens = Tensor::new(tokens.get_ids(), &device)?.unsqueeze(0)?;

    Ok(tokens)
}
