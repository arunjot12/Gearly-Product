-- up.sql
ALTER TABLE products
ADD CONSTRAINT fk_product_shopkeeper
FOREIGN KEY (shopkeeper_id)
REFERENCES shopkeepers(id);