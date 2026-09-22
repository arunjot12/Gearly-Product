-- down.sql
ALTER TABLE products
DROP FOREIGN KEY fk_product_shopkeeper,
DROP COLUMN shopkeeper_id;
