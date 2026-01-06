use yew::prelude::*;
use yew_router::prelude::Link;
use crate::Route;

use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen_futures::spawn_local;
use shared::RoundStatusResponse;

fn api(path: &str) -> String {
    format!("/api{}", path)
}

fn get_session_id_from_url_or_storage() -> String {
    let from_url = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default()
        .replace("?session_id=", "")
        .replace("session_id=", "");

    if !from_url.trim().is_empty() {
        return from_url;
    }
    LocalStorage::get::<String>("session_id").unwrap_or_default()
}

#[function_component(AdminPage)]
pub fn admin_page() -> Html {
    let session_id = use_memo((), |_| get_session_id_from_url_or_storage());

    let status = use_state(|| None::<RoundStatusResponse>);
    let error = use_state(|| "".to_string());
    let loading = use_state(|| false);
    let action_msg = use_state(|| "".to_string());

    let refresh_status: Callback<()> = {
        let status = status.clone();
        let error = error.clone();
        let loading = loading.clone();
        let session_id = (*session_id).clone();

        Callback::from(move |_| {
            let status = status.clone();
            let error = error.clone();
            let loading = loading.clone();
            let session_id = session_id.clone();

            if session_id.trim().is_empty() {
                error.set("No session. Please log in as admin.".into());
                status.set(None);
                return;
            }

            loading.set(true);
            error.set("".into());

            spawn_local(async move {
                let url = format!("{}?session_id={}", api("/admin/status"), session_id);
                match Request::get(&url).send().await {
                    Ok(resp) => match resp.json::<Option<RoundStatusResponse>>().await {
                        Ok(Some(s)) => status.set(Some(s)),
                        Ok(None) => {
                            status.set(None);
                            error.set("Not authorized or no status.".into());
                        }
                        Err(e) => error.set(format!("Parse error: {}", e)),
                    },
                    Err(e) => error.set(format!("Request error: {:?}", e)),
                }
                loading.set(false);
            });
        })
    };

    let on_refresh_click: Callback<MouseEvent> = {
        let refresh_status = refresh_status.clone();
        Callback::from(move |_| refresh_status.emit(()))
    };

    {
        let refresh_status = refresh_status.clone();
        use_effect_with((), move |_| {
            refresh_status.emit(());
            || ()
        });
    }

    let start_round: Callback<()> = {
        let error = error.clone();
        let action_msg = action_msg.clone();
        let session_id = (*session_id).clone();
        let refresh_status = refresh_status.clone();

        Callback::from(move |_| {
            let error = error.clone();
            let action_msg = action_msg.clone();
            let session_id = session_id.clone();
            let refresh_status = refresh_status.clone();

            if session_id.trim().is_empty() {
                error.set("No session. Please log in as admin.".into());
                return;
            }

            error.set("".into());
            action_msg.set("".into());

            spawn_local(async move {
                let url = format!("{}?session_id={}", api("/admin/round/start"), session_id);
                match Request::post(&url).send().await {
                    Ok(resp) if resp.status() == 200 => {
                        action_msg.set("Round started".into());
                        refresh_status.emit(());
                    }
                    Ok(resp) => error.set(format!("Start failed: HTTP {}", resp.status())),
                    Err(e) => error.set(format!("Request error: {:?}", e)),
                }
            });
        })
    };

    let on_start_click: Callback<MouseEvent> = {
        let start_round = start_round.clone();
        Callback::from(move |_| start_round.emit(()))
    };

    let close_round: Callback<()> = {
        let error = error.clone();
        let action_msg = action_msg.clone();
        let session_id = (*session_id).clone();
        let refresh_status = refresh_status.clone();

        Callback::from(move |_| {
            let error = error.clone();
            let action_msg = action_msg.clone();
            let session_id = session_id.clone();
            let refresh_status = refresh_status.clone();

            if session_id.trim().is_empty() {
                error.set("No session. Please log in as admin.".into());
                return;
            }

            error.set("".into());
            action_msg.set("".into());

            spawn_local(async move {
                let url = format!("{}?session_id={}", api("/admin/round/close"), session_id);
                match Request::post(&url).send().await {
                    Ok(resp) if resp.status() == 200 => {
                        action_msg.set("Round closed".into());
                        refresh_status.emit(());
                    }
                    Ok(resp) => error.set(format!("Close failed: HTTP {}", resp.status())),
                    Err(e) => error.set(format!("Request error: {:?}", e)),
                }
            });
        })
    };

    let on_close_click: Callback<MouseEvent> = {
        let close_round = close_round.clone();
        Callback::from(move |_| close_round.emit(()))
    };

    let session_id_str = (*session_id).clone();

    html! {
        <div class="dashboard-common">
            <h1>{ "Admin Panel" }</h1>

            if session_id_str.trim().is_empty() {
                <div class="error-message">
                    <p>{ "No session ID found. Log in as admin first." }</p>
                    <Link<Route> to={Route::Login} classes="btn btn-primary">{ "Go to Login" }</Link<Route>>
                </div>
            } else {
                <div class="info-card">
                    <p style="opacity:0.9;">{ "mail: admin@system" }</p>
                </div>

                <div class="preferences-section">
                    <h3>{ "Round status" }</h3>

                    if *loading {
                        <div class="loading">{ "Loading" }</div>
                    } else if let Some(s) = (*status).clone() {
                        <p>
                            <strong>{ "Round:" }</strong> { s.round_number }
                        </p>
                        <p>
                            <strong>{ "Open:" }</strong> { if s.round_open { "YES" } else { "NO" } }
                        </p>
                    } else {
                        <p><i>{ "No status (maybe not admin?)" }</i></p>
                    }

                    <div class="controls" style="margin-top:15px;">
                        <button class="btn refresh-btn" onclick={on_refresh_click}>{ "Refresh status" }</button>
                    </div>
                </div>

                <div class="preferences-section">
                    <h3>{ "Actions" }</h3>

                    <div class="controls">
                        <button class="btn btn-success" onclick={on_start_click}>{ "Start new round" }</button>
                        <button class="btn btn-danger" onclick={on_close_click}>{ "Close round" }</button>
                        <Link<Route> to={Route::MatchPage} classes="btn btn-primary">{ "See Matches" }</Link<Route>>
                    </div>

                    if !action_msg.is_empty() {
                        <div class="summary" style="margin-top:15px;">
                            { (*action_msg).clone() }
                        </div>
                    }

                    if !error.is_empty() {
                        <div class="error-message" style="margin-top:15px;">
                            { (*error).clone() }
                        </div>
                    }
                </div>

                <div class="navigation">
                    <Link<Route> to={Route::Home} classes="btn back-btn">{ "Back" }</Link<Route>>
                </div>
            }
        </div>
    }
}
