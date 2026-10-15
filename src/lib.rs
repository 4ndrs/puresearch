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

pub fn index_file(
    connection: &mut SqliteConnection,
    hash: &blake3::Hash,
    path: &str,
) -> QueryResult<()> {
    use schema::{files, images};

    connection.transaction(|connection| {
        insert_into(images::table)
            .values(images::blake3.eq(hash.as_bytes()))
            .on_conflict(images::blake3)
            .do_nothing()
            .execute(connection)?;

        let image_id: i32 = images::table
            .filter(images::blake3.eq(hash.as_bytes()))
            .select(images::id)
            .first(connection)?;

        insert_into(files::table)
            .values((files::path.eq(path), files::dsl::image_id.eq(image_id)))
            .on_conflict(files::path)
            .do_update()
            .set(files::image_id.eq(image_id))
            .execute(connection)?;

        Ok(())
    })
}
