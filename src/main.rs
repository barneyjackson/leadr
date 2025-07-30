use leadr_api::{create_app, db};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:./leadr.db".to_string());

    let pool = db::create_pool(&database_url).await?;
    db::run_migrations(&pool).await?;

    let app = create_app(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
