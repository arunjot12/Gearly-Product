use crate::{
    AppState,
     model::{NewProduct, NewProductRequest, Product, UpdateProduct},
    product::handler::{delete_product_db, handle_product, handle_product_insertion, handle_products, update_product_db},
      auth::auth::Claims,
};
use axum::{Json, Extension,extract::{State,Path}, http::StatusCode};

#[axum::debug_handler]
pub async fn create_part(
    State(state): State<AppState>,
     Extension(claims): Extension<Claims>,
    Json(payload): Json<NewProductRequest>,
) -> Result<(StatusCode, String), (StatusCode, String)> {
    
    let connection = state
        .db_pool
        .get()
        .await
        .expect("Failed to get DB connection from pool");

    if payload.price <= 0 {
        return Err((StatusCode::BAD_REQUEST, "Add a valid price".to_string()));
    }

    let payload = NewProduct {
    name: payload.name,
    price: payload.price,
    descri: payload.descri,
    part_number: payload.part_number,
    shopkeeper_id: claims.sub,
    };

    let result = connection
        .interact(move |connection| handle_product_insertion(connection, payload))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match result {
        Ok(_) => Ok((
            StatusCode::CREATED,
            "successfully created product".to_string(),
        )),
        Err(err) => Err((StatusCode::BAD_REQUEST, err.to_string())),
    }
}

#[axum::debug_handler]
pub async fn get_products(State(state): State<AppState>) -> Result<Json<Vec<Product>>, String> {
    let connection = state
        .db_pool
        .get()
        .await
        .expect("Failed to get DB connection from pool");

    let result = connection
        .interact(move |connection| handle_products(connection))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));

    let result = match result {
        Ok(result) => result,
        Err(e) => return Err("BAD_REQUEST".to_string()),
    };
    result
}

#[axum::debug_handler]
pub async fn get_product(State(state): State<AppState>, Json(payload): Json<i32>) -> Result<Json<Product>, String> {
    let connection = state
        .db_pool
        .get()
        .await
        .expect("Failed to get DB connection from pool");

    let result = connection
        .interact(move |connection| handle_product(connection,payload))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()));

    let result = match result {
        Ok(result) => result,
        Err(e) => return Err("BAD_REQUEST".to_string()),
    };
    result
}

#[axum::debug_handler]
pub async fn delete_product(State(state): State<AppState>, Json(payload): Json<i32>) -> Result<StatusCode, (StatusCode, String)>  {
    let connection = state
        .db_pool
        .get()
        .await
        .expect("Failed to get DB connection from pool");

     let result = connection
        .interact(move |connection| delete_product_db(connection, payload))
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database task failed: {e}"),
            )
        })?
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not delete product: {e}"),
            )
        })?;

     if result == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "Product not found".to_string(),
        ));
    }

    Ok(StatusCode::NO_CONTENT)
}

#[axum::debug_handler]
pub async fn update_product(
    State(state): State<AppState>,
    Path(product_id): Path<i32>,
    Json(payload): Json<UpdateProduct>
) -> Result<(StatusCode, String), (StatusCode, String)> {

     let connection = state
        .db_pool
        .get()
        .await
        .expect("Failed to get DB connection from pool");

    let result = connection
        .interact(move |connection| update_product_db(connection, &product_id,payload))
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

   let updated_rows = match result {
        Ok(rows) => rows,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                e.to_string(),
            ));
        }
    };

    if updated_rows == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "Product not found".to_string(),
        ));
    }

    Ok((
        StatusCode::OK,
        "Product updated successfully".to_string(),
    ))
}