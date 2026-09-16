use chrono::NaiveDateTime;
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Queryable, Selectable,Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::products)]
pub struct NewProductRequest{
     pub name: String,
     pub price: i32,
     pub descri: String,
     pub part_number : String,
}

#[derive(Debug, Queryable, Selectable,Serialize, Deserialize, Insertable)]
#[diesel(table_name = crate::schema::products)]
pub struct NewProduct{
     pub name: String,
     pub price: i32,
     pub descri: String,
     pub part_number : String,
     pub shopkeeper_id: i32
}

#[derive(Debug, Queryable, Selectable, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::products)]
pub struct Product {
     pub id: i32,
     pub name: String,
     pub price: i32,
     pub descri: String,
     pub part_number : String,
     pub created_at: Option<NaiveDateTime>,
     pub updated_at: Option<NaiveDateTime>,
     pub shopkeeper_id: i32

}

#[derive(Deserialize)]
pub struct UpdateProduct {
    pub name: String,
    pub price: i32,
    pub descri: String,
    pub part_number: String,
    pub shopkeeper_id: i32
}