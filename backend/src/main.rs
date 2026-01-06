mod matching;
mod models;
mod routes;
mod state;

use axum::Router;
use axum::routing::{get, post};
use std::{
    fs,
    sync::{Arc, Mutex},
};
use tower_http::cors::{Any, CorsLayer};

use state::AppState;

const ADMIN_EMAIL: &str = "admin@system";
const ADMIN_PASSWORD: &str = "admin";

#[tokio::main]
async fn main() {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let state = Arc::new(Mutex::new(
        if let Ok(data) = fs::read_to_string("state.json") {
            serde_json::from_str(&data).unwrap_or_else(|_| AppState::new())
        } else {
            AppState::new()
        },
    ));

    {
        let mut s = state.lock().unwrap();
        if !s.users.iter().any(|u| u.email == ADMIN_EMAIL) {
            s.users.push(models::AuthUser {
                email: ADMIN_EMAIL.into(),
                password: ADMIN_PASSWORD.into(),
                role: models::Role::Admin,
            });
            let _ = s.save();
        }
    }

    let api = Router::new()
        .route("/group", post(routes::add_group))
        .route("/company", post(routes::add_company))
        .route("/login", post(routes::login))
        .route("/group/me", get(routes::group_me))
        .route("/company/me", get(routes::company_me))
        .route("/company/list", get(routes::list_companies))
        .route("/group/list", get(routes::list_groups))
        .route("/projects", get(routes::list_projects))
        .route("/company/projects", post(routes::company_add_project))
        .route("/group/preferences", post(routes::set_group_preferences))
        .route(
            "/project/preferences",
            post(routes::set_project_preferences),
        )
        .route("/project/preferences", get(routes::get_project_preferences))
        .route("/group/preferences", get(routes::get_group_preferences))
        .route("/match", get(routes::match_groups))
        .route("/me/match", get(routes::me_match))
        .route("/me/final", get(routes::me_final))
        .route("/match/accept", post(routes::match_accept))
        .route("/match/reject", post(routes::match_reject))
        .route("/round/status", get(routes::round_status))
        .route("/admin/status", get(routes::admin_status))
        .route("/admin/round/start", post(routes::admin_round_start))
        .route("/admin/round/close", post(routes::admin_round_close));

    let app = Router::new()
        .nest("/api", api)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, app).await.unwrap();
}
