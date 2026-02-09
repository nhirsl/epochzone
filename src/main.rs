use epochzone::routes::create_router;
use shuttle_axum::ShuttleAxum;

#[shuttle_runtime::main]
async fn main() -> ShuttleAxum {
    let app = create_router();
    Ok(app.into())
}
