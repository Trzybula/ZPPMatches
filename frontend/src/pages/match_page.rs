use yew::prelude::*;
use gloo_net::http::Request;
use yew_router::prelude::Link;
use crate::Route;
use shared::MatchResult;

fn api(path: &str) -> String {
    format!("/api{}", path)
}

#[function_component(MatchPage)]
pub fn match_page() -> Html {
    let results = use_state(|| Vec::<MatchResult>::new());
    let loading = use_state(|| true);
    let error = use_state(|| "".to_string());

    {
        let results = results.clone();
        let loading = loading.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                error.set("".to_string());

                match Request::get(&api("/match")).send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            match resp.json::<Vec<MatchResult>>().await {
                                Ok(data) => results.set(data),
                                Err(e) => error.set(format!("Failed to parse: {}", e)),
                            }
                        } else {
                            error.set(format!("Server error: {}", resp.status()));
                        }
                    }
                    Err(e) => {
                        error.set(format!("Network error: {}", e));
                    }
                }

                loading.set(false);
            });

            || ()
        });
    }

    let refresh_matches = {
        let results = results.clone();
        let loading = loading.clone();
        let error = error.clone();

        Callback::from(move |_| {
            let results = results.clone();
            let loading = loading.clone();
            let error = error.clone();

            loading.set(true);
            error.set("".to_string());

            wasm_bindgen_futures::spawn_local(async move {
                match Request::get(&api("/match")).send().await {
                    Ok(resp) => {
                        if resp.ok() {
                            match resp.json::<Vec<MatchResult>>().await {
                                Ok(data) => results.set(data),
                                Err(e) => error.set(format!("Failed to parse: {}", e)),
                            }
                        } else {
                            error.set(format!("Server error: {}", resp.status()));
                        }
                    }
                    Err(e) => {
                        error.set(format!("Network error: {}", e));
                    }
                }

                loading.set(false);
            });
        })
    };

    html! {
        <div class="match-page">
            <div class="page-header">
                <h1>{"Matching Results"}</h1>
                <p class="subtitle">{"Groups matched with projects (company email)"}</p>
            </div>

            <div class="controls">
                <button
                    onclick={refresh_matches}
                    class="btn refresh-btn"
                    disabled={*loading}
                >
                    { if *loading { "Loading..." } else { "Refresh" } }
                </button>

                <Link<Route> to={Route::Home} classes="btn back-btn">
                    {"Home"}
                </Link<Route>>
            </div>

            if *loading {
                <div class="loading-state">
                    <p>{"Loading matches..."}</p>
                </div>
            } else if !error.is_empty() {
                <div class="error-state">
                    <p><strong>{ "Error:" }</strong></p>
                    <p>{ &*error }</p>
                </div>
            } else if results.is_empty() {
                <div class="empty-state">
                    <p>{"No matches yet."}</p>
                    <p>{"Add preferences and try again."}</p>
                </div>
            } else {
                <div class="results-container">
                    <div class="results-header">
                        <h2>{ format!("Found {} matches", results.len()) }</h2>
                        <p class="algorithm-info">{ "group_email → project (company email)" }</p>
                    </div>

                    <div class="matches-list">
                        { for results.iter().enumerate().map(|(i, m)| html! {
                            <div
                                class="match-item"
                                key={format!("{}-{}", m.group_email, m.project_id)}
                            >
                                <div class="match-number">{ i + 1 }</div>
                                <div class="match-details">
                                    <span class="group">{ &m.group_email }</span>
                                    <span class="connector">{ " ⇆ " }</span>
                                    <span class="company">{ &m.project_name }</span>
                                    <span class="email">{ " (mail: " }</span>
                                    <span class="company">{ &m.company_email }</span>
                                    <span class="email">{ ")" }</span>
                                </div>
                                <div class="match-status">{ "✔️" }</div>
                            </div>
                        }) }
                    </div>

                    <div class="summary">
                        <p>{ format!("Total matches: {}", results.len()) }</p>
                    </div>
                </div>
            }
        </div>
    }
}
