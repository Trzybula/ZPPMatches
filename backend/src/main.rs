mod state;
mod models;
mod matching;
mod routes;

use tower_http::cors::{CorsLayer, Any};
use axum::{
    routing::{post, get},
    Router,
};
use state::AppState;
use std::sync::{Arc, Mutex};
use std::fs;

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = Arc::new(Mutex::new(
        if let Ok(data) = fs::read_to_string("state.json") {
            serde_json::from_str(&data).unwrap_or_else(|e| {
                println!("Error loading state: {}, creating new state", e);
                AppState::new()
            })
        } else {
            println!("No state.json found, creating new state");
            AppState::new()
        }
    ));

    let app = Router::new()
        .route("/group", post(routes::add_group))
        .route("/company", post(routes::add_company))
        .route("/match", get(routes::match_groups))
        .route("/login/group", post(routes::login_group))
        .route("/login/company", post(routes::login_company))
        .route("/group/me", get(routes::group_me))
        .route("/company/me", get(routes::company_me))
        .route("/company/list", get(routes::list_companies))
        .route("/group/list", get(routes::list_groups))
        .route("/group/add_pref", post(routes::group_add_pref))
        .route("/company/add_pref", post(routes::company_add_pref))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Running backend on http://localhost:3000/");
    
    axum::serve(listener, app).await.unwrap();
}
