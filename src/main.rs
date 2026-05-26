mod state;
mod models;
mod dto;
mod handlers;
mod routes;
mod middleware;
mod services;
mod utils;

use axum::{middleware::from_fn_with_state, Router};
use dotenvy::dotenv;
use sqlx::mysql::MySqlPoolOptions;
use std::env;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use state::AppState;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // 1. Ambil PORT dari Google Cloud dan langsung satukan ke format socket address
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let addr_str = format!("0.0.0.0:{}", port);
    
    // Gunakan std::net::TcpListener bawaan atau langsung bind string-nya
    let listener = tokio::net::TcpListener::bind(&addr_str)
        .await
        .expect("Failed to bind port");

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rakshaka=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let jwt_secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set");
    let link_checker_api_key = env::var("LINK_CHECKER_API_KEY").unwrap_or_default();
    let link_checker_api_url = env::var("LINK_CHECKER_API_URL").unwrap_or_default();

    // 2. Koneksi ke Database Aiven
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
        .expect("Failed to connect to the database");

    tracing::info!("Database connected successfully");

    // 3. Jalankan Migrasi setelah port siap
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    tracing::info!("Migrations applied");

    let state = AppState {
        db: pool,
        jwt_secret,
        link_checker_api_key,
        link_checker_api_url,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Protected routes
    let protected = Router::new()
        .merge(routes::auth::protected_routes())
        .merge(routes::reports::protected_routes())
        .merge(routes::link::protected_routes())
        .route_layer(from_fn_with_state(
            state.clone(),
            middleware::auth::require_auth,
        ));

    // Public routes
    let public = routes::auth::public_routes();

    let app = Router::new()
        .merge(protected)
        .merge(public)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state);

    tracing::info!("Server running at http://{}", addr_str);

    // 4. Jalankan server Axum dengan passing ownership listener secara bersih
    axum::serve(listener, app)
        .await
        .expect("Server error");
}