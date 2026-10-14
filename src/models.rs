use diesel::prelude::*;

use crate::schema::{files, images};

#[derive(Debug, HasQuery)]
#[diesel(table_name = images)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Image {
    id: i32,
    blake3: Vec<u8>,
}

#[derive(Debug, HasQuery, Associations)]
#[diesel(belongs_to(Image))]
#[diesel(table_name = files)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct File {
    id: i32,
    path: String,
    image_id: i32,
}
