mod models;
mod schema;

use diesel::insert_into;
use dotenvy::dotenv;

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
