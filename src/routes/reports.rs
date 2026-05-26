use axum::{routing::get, Router};

use crate::{
    handlers::reports::{create_report, delete_report, get_report, list_reports, update_report},
    state::AppState,
};

pub fn protected_routes() -> Router<AppState> {
    Router::new()
        .route("/reports", get(list_reports).post(create_report))
        .route(
            "/reports/{id}",
            get(get_report).put(update_report).delete(delete_report),
        )
}
