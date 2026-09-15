-- Your SQL goes here
ALTER TABLE shopkeepers
ADD COLUMN last_name VARCHAR(255) NOT NULL AFTER first_name;