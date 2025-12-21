use yew::prelude::*;
use gloo_net::http::Request;
use serde::Deserialize;
use yew_router::prelude::Link;
use crate::Route;

#[derive(Deserialize, Clone, Debug)]
pub struct MatchResult {
    pub group: String,
    pub company: String,
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
                match Request::get("http://localhost:3000/match").send().await {
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
                match Request::get("http://localhost:3000/match").send().await {
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
                <p class="subtitle">{"Groups matched with companies"}</p>
            </div>

            <div class="controls">
                <button onclick={refresh_matches} class="btn refresh-btn">
                    {"Refresh"}
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
                    <p>{ "Error:" }</p>
                    <p>{ &*error }</p>
                </div>
            } else if results.is_empty() {
                <div class="empty-state">
                    <p>{"No matches found yet."}</p>
                    <p>{"Add some groups and companies first!"}</p>
                </div>
            } else {
                <div class="results-container">
                    <div class="results-header">
                        <h2>{ format!("Found {} matches", results.len()) }</h2>
                        <p class="algorithm-info">{"Algorithm: Gale-Shapley"}</p>
                    </div>

                    <div class="matches-list">
                        { for results.iter().enumerate().map(|(i, m)| html! {
                            <div class="match-item" key={i}>
                                <div class="match-number">{ i + 1 }</div>
                                <div class="match-details">
                                    <span class="group">{ &m.group }</span>
                                    <span class="connector">{" ⇆ "}</span>
                                    <span class="company">{ &m.company }</span>
                                </div>
                                <div class="match-status">{"MATCHED"}</div>
                            </div>
                        }) }
                    </div>

                    <div class="summary">
                        <p>{ "Each group is matched with their most preferred available company." }</p>
                    </div>
                </div>
            }
        </div>
    }
}
