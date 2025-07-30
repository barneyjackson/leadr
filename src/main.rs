use leadr_api::{create_app, db};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))
        .init();

    // Validate required environment variables on startup
    std::env::var("LEADR_API_KEY").unwrap_or_else(|_| {
        eprintln!("ERROR: LEADR_API_KEY environment variable is required but not set");
        eprintln!("Please set it before starting the server:");
        eprintln!("  export LEADR_API_KEY=\"your-secret-api-key\"");
        eprintln!("  cargo run");
        std::process::exit(1);
    });

    // Initialize database with proper lifecycle management
    let pool = db::initialize_database().await?;

    let app = create_app(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    tracing::info!("Server listening on http://0.0.0.0:3000");

    axum::serve(listener, app).await?;

    Ok(())
}
