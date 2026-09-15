-- Your SQL goes here
ALTER TABLE products
ADD CONSTRAINT fk_product_shopkeeper
FOREIGN KEY (id)
REFERENCES shopkeepers(id);