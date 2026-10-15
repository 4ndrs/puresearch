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
) -> Result<(), Box<dyn std::error::Error>> {
    use schema::files;
    use schema::images;

    connection.transaction(|connection| {
        insert_into(images::dsl::images)
            .values(images::dsl::blake3.eq(hash.as_bytes()))
            .on_conflict(images::dsl::blake3)
            .do_nothing()
            .execute(connection)?;

        let image_id: i32 = images::dsl::images
            .filter(images::dsl::blake3.eq(hash.as_bytes()))
            .select(images::dsl::id)
            .get_result(connection)?;

        insert_into(files::dsl::files)
            .values((files::dsl::path.eq(path), files::dsl::image_id.eq(image_id)))
            .on_conflict(files::dsl::path)
            .do_update()
            .set(files::dsl::image_id.eq(image_id))
            .execute(connection)?;

        Ok::<(), Box<dyn std::error::Error>>(())
    })?;

    Ok(())
}
