use yew::prelude::*;
use yew_router::prelude::Link;
use crate::Route;
use gloo_net::http::Request;
use serde::{Serialize, Deserialize};
use wasm_bindgen_futures::spawn_local;
use web_sys::HtmlInputElement;

#[derive(Deserialize, Clone, Debug)]
pub struct Company {
    pub name: String,
    pub email: String,
    pub password: String,
    pub preferences: Vec<String>,
}

#[derive(Deserialize, Clone, Debug)]
pub struct Group {
    pub name: String,
    pub email: String,
    pub password: String,
    pub preferences: Vec<String>,
}

#[derive(Serialize)]
struct AddPrefRequest {
    session_id: String,
    pref: String,
}

#[function_component(DashboardCompanyPage)]
pub fn dashboard_company_page() -> Html {
    let company = use_state(|| None::<Company>);
    let groups = use_state(|| Vec::<Group>::new());
    let new_pref = use_state(|| "".to_string());
    let error = use_state(|| "".to_string());

    let session_id = web_sys::window()
        .unwrap().location().search().unwrap_or_default()
        .replace("?session_id=", "");

    let refresh_company = {
        let company = company.clone();
        let session_id = session_id.clone();
        
        Callback::from(move |_| {
            let company = company.clone();
            let session_id = session_id.clone();
            
            spawn_local(async move {
                let url = format!("http://localhost:3000/company/me?session_id={}", session_id);
                if let Ok(resp) = Request::get(&url).send().await {
                    if let Ok(data) = resp.json::<Option<Company>>().await {
                        company.set(data);
                    }
                }
            });
        })
    };
    {
        let refresh_company = refresh_company.clone();
        use_effect_with((), move |_| {
            refresh_company.emit(());
            || ()
        });
    }
    {
        let groups = groups.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get("http://localhost:3000/group/list").send().await {
                    if let Ok(list) = resp.json::<Vec<Group>>().await {
                        groups.set(list);
                    }
                }
            });
            || ()
        });
    }

    let on_add_pref = {
        let company = company.clone();
        let new_pref = new_pref.clone();
        let error = error.clone();
        let session_id = session_id.clone();
        let groups = groups.clone();
        let refresh_company = refresh_company.clone();

        Callback::from(move |_| {
            let pref = (*new_pref).trim().to_string();

            if pref.is_empty() {
                error.set("Preference cannot be empty".into());
                return;
            }

            if !groups.iter().any(|g| g.name == pref) {
                error.set(format!("Group '{}' does not exist", pref));
                return;
            }

            if let Some(c) = &*company {
                if c.preferences.contains(&pref) {
                    error.set(format!("Group '{}' is already in preferences", pref));
                    return;
                }
            }

            new_pref.set("".into());
            error.set("".into());

            let session = session_id.clone();
            let refresh_company = refresh_company.clone();
            
            spawn_local(async move {
                let request = AddPrefRequest {
                    session_id: session,
                    pref: pref.clone(),
                };
                let result = Request::post("http://localhost:3000/company/add_pref")
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&request).unwrap())
                    .expect("Failed to create request")
                    .send()
                    .await;

                match result {
                    Ok(resp) => {
                        if resp.status() == 200 {
                            refresh_company.emit(());
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
        <div class="dashboard-common dashboard-company">
            <h1>{ "Company Dashboard" }</h1>

            { if let Some(c) = (*company).clone() {
                html!{
                    <>
                        <div class="info-card">
                            <p><strong>{ "Name:" }</strong> { &c.name }</p>
                            <p><strong>{ "Email:" }</strong> { &c.email }</p>
                        </div>

                        <div class="preferences-section">
                            <h3>{ "Preferences" }</h3>
                            if c.preferences.is_empty() {
                                <p><i>{ "No preferences added yet" }</i></p>
                            } else {
                                <ul class="preferences-list">
                                    { for c.preferences.iter().map(|p| html!{ 
                                        <li key={p.clone()}>
                                            <span>{p}</span>
                                        </li>
                                    }) }
                                </ul>
                            }
                        </div>

                        <div class="preferences-section">
                            <h3>{ "Add Preference" }</h3>
                            <div class="input-group">
                                <input
                                    type="text"
                                    value={(*new_pref).clone()}
                                    placeholder="Enter group name"
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
                            <h3>{ "Available Groups" }</h3>
                            if groups.is_empty() {
                                <p><i>{ "No groups available" }</i></p>
                            } else {
                                <ul class="list-items">
                                    { for groups.iter().map(|g| html!{ 
                                        <li key={g.name.clone()}>
                                            <span>{ &g.name }</span>
                                            { if c.preferences.contains(&g.name) {
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
                                { "See Matches" }
                            </Link<Route>>
                        </div>
                    </>
                }
            } else if session_id.is_empty() {
                html!{ 
                    <div class="error-message">
                        <p>{ "No session ID found. Please log in again." }</p>
                        <Link<Route> to={Route::LoginCompany}>{ "Go to Login" }</Link<Route>>
                    </div>
                }
            } else {
                html!{ 
                    <div class="loading">
                        <p>{"Loading..."}</p>
                    </div>
                }
            }}
        </div>
    }
}