// @generated automatically by Diesel CLI.

diesel::table! {
    files (id) {
        id -> Integer,
        path -> Text,
        image_id -> Integer,
    }
}

diesel::table! {
    images (id) {
        id -> Integer,
        blake3 -> Binary,
        embedding -> Nullable<Binary>,
    }
}

diesel::joinable!(files -> images (image_id));

diesel::allow_tables_to_appear_in_same_query!(files, images,);
