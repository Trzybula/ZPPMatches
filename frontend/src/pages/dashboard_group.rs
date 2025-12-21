use yew::prelude::*;
use yew_router::prelude::Link;
use crate::Route;
use gloo_net::http::Request;
use serde::{Serialize, Deserialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Group {
    pub name: String,
    pub email: String,
    pub password: String,
    pub preferences: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Company {
    pub name: String,
    pub email: String,
    pub password: String,
    pub preferences: Vec<String>,
}

#[derive(Serialize, Deserialize)]
struct AddPrefRequest {
    pub session_id: String,
    pub pref: String,
}

#[function_component(DashboardGroupPage)]
pub fn dashboard_group_page() -> Html {
    let group = use_state(|| None::<Group>);
    let companies = use_state(|| Vec::<Company>::new());
    let new_pref = use_state(|| "".to_string());
    let error = use_state(|| "".to_string());

    let session_id = web_sys::window()
        .unwrap().location().search().unwrap_or_default()
        .replace("?session_id=", "");

    let refresh_group = {
        let group = group.clone();
        let session_id = session_id.clone();
        
        Callback::from(move |_| {
            let group = group.clone();
            let session_id = session_id.clone();
            
            spawn_local(async move {
                let url = format!("http://localhost:3000/group/me?session_id={}", session_id);
                if let Ok(resp) = Request::get(&url).send().await {
                    if let Ok(data) = resp.json::<Option<Group>>().await {
                        group.set(data);
                    }
                }
            });
        })
    };

    {
        let refresh_group = refresh_group.clone();
        use_effect_with((), move |_| {
            refresh_group.emit(());
            || ()
        });
    }

    {
        let companies = companies.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3000/company/list").send().await {
                    if let Ok(list) = resp.json::<Vec<Company>>().await {
                        companies.set(list);
                    }
                }
            });
            || ()
        });
    }
    let on_add_pref = {
        let group = group.clone();
        let new_pref = new_pref.clone();
        let error = error.clone();
        let companies = companies.clone();
        let session_id = session_id.clone();
        let refresh_group = refresh_group.clone();

        Callback::from(move |_| {
            let pref = (*new_pref).trim().to_string();

            if pref.is_empty() {
                error.set("Preference cannot be empty".into());
                return;
            }

            if !companies.iter().any(|c| c.name == pref) {
                error.set(format!("Company '{}' does not exist!", pref));
                return;
            }
            if let Some(g) = &*group {
                if g.preferences.contains(&pref) {
                    error.set(format!("Company '{}' is already in preferences", pref));
                    return;
                }
            }

            new_pref.set("".into());
            error.set("".into());

            let session = session_id.clone();
            let refresh_group = refresh_group.clone();
            
            spawn_local(async move {
                let request = AddPrefRequest {
                    session_id: session,
                    pref: pref.clone(),
                };

                let result = Request::post("http://localhost:3000/group/add_pref")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&request).unwrap())
                    .expect("Failed to create request")
                    .send()
                    .await;

                match result {
                    Ok(resp) => {
                        if resp.status() == 200 {
                            refresh_group.emit(());
                        } else {
                            web_sys::console::error_1(&"Failed to add preference".into());
                        }
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Request error: {:?}", e).into());
                    }
                }
            });
        })
    };

    let on_input = {
        let error = error.clone();
        let new_pref = new_pref.clone();
        
        Callback::from(move |e: InputEvent| {
            let input = e.target_unchecked_into::<HtmlInputElement>();
            new_pref.set(input.value());
            if !error.is_empty() {
                error.set("".into());
            }
        })
    };

    html! {
        <div class="dashboard-common dashboard-group">
            <h1>{ "Group Dashboard" }</h1>

            { if let Some(g) = (*group).clone() {
                html!{
                    <>
                        <div class="info-card">
                            <p><strong>{ "Name:" }</strong> { &g.name }</p>
                            <p><strong>{ "Email:" }</strong> { &g.email }</p>
                        </div>

                        <div class="preferences-section">
                            <h3>{ "Your Preferences" }</h3>
                            if g.preferences.is_empty() {
                                <p><i>{ "No preferences added yet. Add companies you're interested in!" }</i></p>
                            } else {
                                <ul class="preferences-list">
                                    { for g.preferences.iter().map(|p| html!{ 
                                        <li key={p.clone()}>
                                            <span>{p}</span>
                                        </li>
                                    }) }
                                </ul>
                            }
                        </div>

                        <div class="preferences-section">
                            <h3>{ "Add Company Preference" }</h3>
                            <div class="input-group">
                                <input
                                    type="text"
                                    value={(*new_pref).clone()}
                                    placeholder="Enter company name"
                                    oninput={on_input}
                                    class={if !error.is_empty() { "input-error" } else { "" }}
                                />
                                <button 
                                    onclick={on_add_pref}
                                    disabled={session_id.is_empty()}
                                    class="btn btn-success"
                                >
                                    { "Add" }
                                </button>
                            </div>
                            if !error.is_empty() {
                                <div class="error-message">
                                    { (*error).clone() }
                                </div>
                            }
                            if session_id.is_empty() {
                                <div class="error-message">
                                    { "No session ID found. Please log in again." }
                                </div>
                            }
                        </div>

                        <div class="available-list">
                            <h3>{ "Available Companies" }</h3>
                            if companies.is_empty() {
                                <p><i>{ "No companies available" }</i></p>
                            } else {
                                <ul class="list-items">
                                    { for companies.iter().map(|c| html!{ 
                                        <li key={c.name.clone()}>
                                            <span>{ &c.name }</span>
                                            { if g.preferences.contains(&c.name) {
                                                html!{ <span class="already-added">{" (added)"}</span> }
                                            } else {
                                                html!{}
                                            }}
                                        </li>
                                    }) }
                                </ul>
                            }
                        </div>

                        <div class="navigation">
                            <Link<Route> 
                                to={Route::MatchPage}
                                classes="btn btn-primary"
                            >
                                { "See Your Matches" }
                            </Link<Route>>
                        </div>
                    </>
                }
            } else if session_id.is_empty() {
                html!{ 
                    <div class="error-message">
                        <p>{ "No session ID found. Please log in again." }</p>
                        <Link<Route> to={Route::Home}>{ "Go to Login" }</Link<Route>>
                    </div>
                }
            } else {
                html!{ 
                    <div class="loading">
                        <p>{"Loading your dashboard..."}</p>
                    </div>
                }
            }}
        </div>
    }
}