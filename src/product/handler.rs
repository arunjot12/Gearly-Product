use crate::{
    model::{NewProduct, UpdateProduct,Product},
    schema::products,
};
use axum::Json;
use diesel::{
    ExpressionMethods, MysqlConnection, OptionalExtension, QueryDsl, QueryResult, RunQueryDsl, SelectableHelper, dsl::insert_into,
};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Part Number PartNumberAlreadyExits")]
    PartNumberAlreadyExits,

    #[error("Database issue")]
    Database(#[from] diesel::result::Error),
}

pub fn handle_product_insertion(
    connection: &mut MysqlConnection,
    part: NewProduct,
) -> Result<String, AppError> {
    let check = product::table
        .select(Product::as_select())
        .filter(product::part_number.eq(&part.part_number))
        .first(connection)
        .optional();

    match check {
        Ok(Some(_)) => return Err(AppError::PartNumberAlreadyExits),
        Ok(None) => {}
        Err(err) => return Err(AppError::Database(err)),
    }

    let insert_into = insert_into(product::table).values(part).execute(connection);

    match insert_into {
        Ok(_) => Ok(" Everything is done bro".to_string()),
        Err(e) => return Err(AppError::Database(e)),
    }
}
pub fn handle_products(
    connection: &mut MysqlConnection,
) -> Result<Json<Vec<Product>>, String> {
    let products = product::table
            .select(Product::as_select())
            .load::<Product>(connection).map_err(|e| e.to_string())?;
    Ok(Json(products))
}

pub fn handle_product(
    connection: &mut MysqlConnection,
    product_id: i32,
) -> Result<Json<Product>, String> {
    let products = 
       product::table
            .select(Product::as_select())
            .filter(product::id.eq(product_id))
            .first::<Product>(connection).map_err(|e| e.to_string())?;

    Ok(Json(products))
}

pub fn delete_product_db(
    connection: &mut MysqlConnection,
    product_id: i32,
) -> QueryResult<usize> {
    diesel::delete(
        product::table.filter(product::id.eq(product_id)),
    )
    .execute(connection)
}

pub fn update_product_db(
    connection: &mut MysqlConnection,
    product_id: &i32,
    payload: UpdateProduct,
) -> QueryResult<usize> {
    diesel::update(product::table.filter(product::id.eq(product_id)))
        .set((
            product::name.eq(payload.name),
            product::price.eq(payload.price),
            product::descri.eq(payload.descri),
            product::part_number.eq(payload.part_number),
        ))
        .execute(connection)
}