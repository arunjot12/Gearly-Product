-- up.sql
ALTER TABLE products
ADD COLUMN shopkeeper_id INT NOT NULL,
ADD CONSTRAINT fk_product_shopkeeper
FOREIGN KEY (shopkeeper_id)
REFERENCES shopkeepers(id);