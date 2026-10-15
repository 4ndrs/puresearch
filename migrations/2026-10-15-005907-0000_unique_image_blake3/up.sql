-- Your SQL goes here
CREATE UNIQUE INDEX images_blake3_unique
ON images (blake3);
