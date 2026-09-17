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
    claims: &i32
) -> Result<String, AppError> {
    let check = products::table
        .select(Product::as_select())
         .filter(products::shopkeeper_id.eq(claims))
        .filter(products::part_number.eq(&part.part_number))
        .first(connection)
        .optional();

    match check {
        Ok(Some(_)) => return Err(AppError::PartNumberAlreadyExits),
        Ok(None) => {}
        Err(err) => return Err(AppError::Database(err)),
    }

    let insert_into = insert_into(products::table).values(part).execute(connection);

    match insert_into {
        Ok(_) => Ok(" Everything is done bro".to_string()),
        Err(e) => Err(AppError::Database(e)),
    }
}

pub fn handle_products(
    connection: &mut MysqlConnection,
    claims: &i32,
    limit: i64,
    offset: i64,
) -> Result<Json<Vec<Product>>, String> {
    let products = products::table
            .select(Product::as_select())
            .filter(products::shopkeeper_id.eq(claims))
             .limit(limit)
            .offset(offset)
            .load::<Product>(connection).map_err(|e| e.to_string())?;
    Ok(Json(products))
}

pub fn handle_product(
    connection: &mut MysqlConnection,
    product_id: i32,
    claims: &i32,
) -> Result<Json<Product>, String> {
    let products = 
       products::table
            .select(Product::as_select())
            .filter(products::id.eq(product_id))
            .filter(products::shopkeeper_id.eq(claims))
            .first::<Product>(connection).map_err(|e| e.to_string())?;

    Ok(Json(products))
}

pub fn delete_product_db(
    connection: &mut MysqlConnection,
    product_id: i32,
    claims: &i32
) -> QueryResult<usize> {
    diesel::delete(
        products::table
        .filter(products::shopkeeper_id.eq(claims))
        .filter(products::id.eq(product_id)),
    )
    .execute(connection)
}

pub fn update_product_db(
    connection: &mut MysqlConnection,
    product_id: &i32,
    payload: UpdateProduct,
    claims: &i32
) -> QueryResult<usize> {
    diesel::update(products::table
        .filter(products::shopkeeper_id.eq(claims))
        .filter(products::id.eq(product_id)))
        .set((
            products::name.eq(payload.name),
            products::price.eq(payload.price),
            products::descri.eq(payload.descri),
            products::part_number.eq(payload.part_number),
        ))
        .execute(connection)
}