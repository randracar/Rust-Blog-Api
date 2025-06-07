CREATE TABLE posts (
    id SERIAL PRIMARY KEY,
    author VARCHAR(255) NOT NULL,
    title TEXT NOT NULL,
    text TEXT NOT NULL,
    created_at TEXT NOT NULL,
    edited BOOLEAN NOT NULL DEFAULT FALSE,
    edited_at TEXT NOT NULL
);
