use axum::{routing::get, Router};

use crate::{
    handlers::public::{get_public_report, list_public_reports},
    state::AppState,
};

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/reports/public", get(list_public_reports))
        .route("/reports/public/{id}", get(get_public_report))
}
