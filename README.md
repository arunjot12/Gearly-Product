# Gearly-Product

This is the product microservice for Gearly.

## Current Progress

- **Framework**: Rust with Axum and Tokio.
- **Database**: MySQL using Diesel ORM and deadpool-diesel for connection pooling.
- **Authentication**: JWT-based authentication middleware is implemented and applied to routes
- **Observability**: Added `tracing` for structured logging.
- **CORS**: Configured cross-origin requests using `tower-http`.

### Implemented Endpoints

- `POST /create_product` - Create a new product. Expects a JSON payload (`name`, `price`, `descri`, `part_number`).
- `GET /get_products` - Retrieve all products for the authenticated shopkeeper.
- `GET /get_product/:id` - Retrieve a specific product by its ID in the path.
- `PUT /update_product/:id` - Update an existing product. Expects JSON payload.
- `POST /delete_product` - Delete a product by its ID (passed in the JSON payload).
- `GET /health` - Health check endpoint.

### Models

The `Product` model includes the following fields:
- `id`
- `name`
- `price`
- `descri`
- `part_number`
- `created_at`
- `updated_at`
- `shopkeeper_id`
