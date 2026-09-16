-- Your SQL goes here
CREATE TABLE product(
    id INT PRIMARY KEY,
    name VarChar(255) NOT NULL,
    price INT NOT NULL,
    descri TEXT,
    part_number VarChar(100),
    created_at TimeStamp ,
    updated_at TimeStamp
)