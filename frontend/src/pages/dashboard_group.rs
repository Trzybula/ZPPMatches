use yew::prelude::*;
use yew_router::prelude::Link;
use crate::Route;
use gloo_net::http::Request;
use wasm_bindgen_futures::spawn_local;
use gloo_storage::{LocalStorage, Storage};
use shared::{
    Company, Group, Project,
    GroupPreferencesResponse, SetGroupPreferencesRequest,
    CurrentMatchResponse, MatchActionRequest,
    RoundStatusResponse,
};

fn api(path: &str) -> String {
    format!("/api{}", path)
}

fn clean_session_id(mut s: String) -> String {
    s = s.trim().to_string();
    while s.ends_with('&') {
        s.pop();
    }
    s
}

fn get_session_id_from_url_or_storage() -> String {
    let from_url = web_sys::window()
        .and_then(|w| w.location().search().ok())
        .unwrap_or_default()
        .replace("?session_id=", "")
        .replace("session_id=", "");

    if !from_url.trim().is_empty() {
        return clean_session_id(from_url);
    }

    clean_session_id(LocalStorage::get::<String>("session_id").unwrap_or_default())
}

#[function_component(DashboardGroupPage)]
pub fn dashboard_group_page() -> Html {
    let group = use_state(|| None::<Group>);
    let projects = use_state(|| Vec::<Project>::new());
    let companies = use_state(|| Vec::<Company>::new());

    let prefs = use_state(|| Vec::<String>::new());
    let error = use_state(|| "".to_string());

    let round_status = use_state(|| None::<RoundStatusResponse>);
    let current_matches = use_state(|| Vec::<CurrentMatchResponse>::new());
    let final_matches = use_state(|| Vec::<CurrentMatchResponse>::new());
    let match_error = use_state(|| "".to_string());
    let match_msg = use_state(|| "".to_string());
    let final_error = use_state(|| "".to_string());

    let session_id = use_state(get_session_id_from_url_or_storage);
    let session_id_str = (*session_id).clone();

    {
        let round_status = round_status.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api("/round/status")).send().await {
                    if let Ok(data) = resp.json::<RoundStatusResponse>().await {
                        round_status.set(Some(data));
                    }
                }
            });
            || ()
        });
    }

    {
        let group = group.clone();
        let session_id = session_id.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let sid = clean_session_id((*session_id).clone());
                if sid.trim().is_empty() {
                    group.set(None);
                    return;
                }
                let url = format!("{}?session_id={}", api("/group/me"), sid);
                if let Ok(resp) = Request::get(&url).send().await {
                    if let Ok(data) = resp.json::<Option<Group>>().await {
                        group.set(data);
                    }
                }
            });
            || ()
        });
    }

    {
        let projects = projects.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api("/projects")).send().await {
                    if let Ok(list) = resp.json::<Vec<Project>>().await {
                        projects.set(list);
                    }
                }
            });
            || ()
        });
    }

    {
        let companies = companies.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api("/company/list")).send().await {
                    if let Ok(list) = resp.json::<Vec<Company>>().await {
                        companies.set(list);
                    }
                }
            });
            || ()
        });
    }

    {
        let prefs = prefs.clone();
        let session_id = session_id.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let sid = clean_session_id((*session_id).clone());
                if sid.trim().is_empty() {
                    prefs.set(vec![]);
                    return;
                }
                let url = format!("{}?session_id={}", api("/group/preferences"), sid);
                if let Ok(resp) = Request::get(&url).send().await {
                    if let Ok(data) = resp.json::<Option<GroupPreferencesResponse>>().await {
                        if let Some(p) = data {
                            prefs.set(p.project_ids_ranked);
                        } else {
                            prefs.set(vec![]);
                        }
                    }
                }
            });
            || ()
        });
    }

    let save_group_prefs = {
        let session_id = session_id.clone();
        Callback::from(move |next: Vec<String>| {
            let session_id = session_id.clone();
            spawn_local(async move {
                let sid = clean_session_id((*session_id).clone());
                if sid.trim().is_empty() {
                    return;
                }
                let req = SetGroupPreferencesRequest {
                    session_id: sid,
                    project_ids_ranked: next,
                };
                let _ = Request::post(&api("/group/preferences"))
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&req).unwrap())
                    .unwrap()
                    .send()
                    .await;
            });
        })
    };

    let on_add_project = {
        let prefs = prefs.clone();
        let error = error.clone();
        let save_group_prefs = save_group_prefs.clone();

        Callback::from(move |project_id: String| {
            if (*prefs).iter().any(|x| x == &project_id) {
                error.set("Project already added.".into());
                return;
            }
            let mut next = (*prefs).clone();
            next.push(project_id);
            prefs.set(next.clone());
            error.set("".into());
            save_group_prefs.emit(next);
        })
    };

    let on_remove_project = {
        let prefs = prefs.clone();
        let save_group_prefs = save_group_prefs.clone();

        Callback::from(move |project_id: String| {
            let next: Vec<String> = (*prefs).iter().cloned().filter(|x| x != &project_id).collect();
            prefs.set(next.clone());
            save_group_prefs.emit(next);
        })
    };

    let refresh_current_matches: Callback<()> = {
        let session_id = session_id.clone();
        let current_matches = current_matches.clone();
        let match_error = match_error.clone();
        let match_msg = match_msg.clone();

        Callback::from(move |_| {
            let session_id = session_id.clone();
            let current_matches = current_matches.clone();
            let match_error = match_error.clone();
            let match_msg = match_msg.clone();

            spawn_local(async move {
                let sid = clean_session_id((*session_id).clone());
                if sid.trim().is_empty() {
                    current_matches.set(vec![]);
                    return;
                }

                match_error.set("".into());
                match_msg.set("".into());

                let url = format!("{}?session_id={}", api("/me/match"), sid);
                match Request::get(&url).send().await {
                    Ok(resp) if resp.status() == 200 => {
                        match resp.json::<Vec<CurrentMatchResponse>>().await {
                            Ok(list) => current_matches.set(list),
                            Err(e) => match_error.set(format!("Parse error: {}", e)),
                        }
                    }
                    Ok(resp) => {
                        current_matches.set(vec![]);
                        match_error.set(format!("Match endpoint returned HTTP {}", resp.status()));
                    }
                    Err(e) => match_error.set(format!("Request error: {:?}", e)),
                }
            });
        })
    };

    {
        let refresh_current_matches = refresh_current_matches.clone();
        use_effect_with((), move |_| {
            refresh_current_matches.emit(());
            || ()
        });
    }

    let refresh_final_matches: Callback<()> = {
        let session_id = session_id.clone();
        let final_matches = final_matches.clone();
        let final_error = final_error.clone();

        Callback::from(move |_| {
            let session_id = session_id.clone();
            let final_matches = final_matches.clone();
            let final_error = final_error.clone();

            spawn_local(async move {
                let sid = clean_session_id((*session_id).clone());
                if sid.trim().is_empty() {
                    final_matches.set(vec![]);
                    return;
                }

                final_error.set("".into());

                let url = format!("{}?session_id={}", api("/me/final"), sid);
                match Request::get(&url).send().await {
                    Ok(resp) if resp.status() == 200 => {
                        match resp.json::<Vec<CurrentMatchResponse>>().await {
                            Ok(list) => final_matches.set(list),
                            Err(e) => final_error.set(format!("Parse error: {}", e)),
                        }
                    }
                    Ok(resp) => final_error.set(format!("Final endpoint returned HTTP {}", resp.status())),
                    Err(e) => final_error.set(format!("Request error: {:?}", e)),
                }
            });
        })
    };

    {
        let refresh_final_matches = refresh_final_matches.clone();
        use_effect_with((), move |_| {
            refresh_final_matches.emit(());
            || ()
        });
    }
    let accept_match = {
        let session_id = session_id.clone();
        let match_error = match_error.clone();
        let match_msg = match_msg.clone();
        let refresh_current_matches = refresh_current_matches.clone();
        let refresh_final_matches = refresh_final_matches.clone();

        Callback::from(move |_| {
            let session_id = session_id.clone();
            let match_error = match_error.clone();
            let match_msg = match_msg.clone();
            let refresh_current_matches = refresh_current_matches.clone();
            let refresh_final_matches = refresh_final_matches.clone();

            spawn_local(async move {
                let sid = clean_session_id((*session_id).clone());
                if sid.trim().is_empty() {
                    return;
                }

                match_error.set("".into());
                match_msg.set("".into());

                let body = MatchActionRequest { session_id: sid, project_id: None };

                match Request::post(&api("/match/accept"))
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&body).unwrap())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(r) if r.status() == 200 => {
                        match_msg.set("Accepted".into());
                        refresh_current_matches.emit(());
                        refresh_final_matches.emit(());
                    }
                    Ok(r) => match_error.set(format!("Accept failed: HTTP {}", r.status())),
                    Err(e) => match_error.set(format!("Request error: {:?}", e)),
                }
            });
        })
    };

    let reject_match = {
        let session_id = session_id.clone();
        let match_error = match_error.clone();
        let match_msg = match_msg.clone();
        let refresh_current_matches = refresh_current_matches.clone();
        let refresh_final_matches = refresh_final_matches.clone();

        Callback::from(move |_| {
            let session_id = session_id.clone();
            let match_error = match_error.clone();
            let match_msg = match_msg.clone();
            let refresh_current_matches = refresh_current_matches.clone();
            let refresh_final_matches = refresh_final_matches.clone();

            spawn_local(async move {
                let sid = clean_session_id((*session_id).clone());
                if sid.trim().is_empty() {
                    return;
                }

                match_error.set("".into());
                match_msg.set("".into());

                let body = MatchActionRequest { session_id: sid, project_id: None };

                match Request::post(&api("/match/reject"))
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&body).unwrap())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(r) if r.status() == 200 => {
                        match_msg.set("Rejected".into());
                        refresh_current_matches.emit(());
                        refresh_final_matches.emit(());
                    }
                    Ok(r) => match_error.set(format!("Reject failed: HTTP {}", r.status())),
                    Err(e) => match_error.set(format!("Request error: {:?}", e)),
                }
            });
        })
    };

    let company_name_by_email = {
        let companies = (*companies).clone();
        move |email: &str| -> String {
            companies
                .iter()
                .find(|c| c.email == email)
                .map(|c| c.name.clone())
                .unwrap_or_else(|| email.to_string())
        }
    };

    let available_projects: Vec<Project> = (*projects)
        .iter()
        .filter(|p| p.active && p.capacity > 0)
        .cloned()
        .collect();

    let ranked_projects: Vec<Project> = (*prefs)
        .iter()
        .filter_map(|id| (*projects).iter().find(|p| &p.id == id).cloned())
        .collect();

    html! {
        <div class="dashboard-common dashboard-group">
            <h1>{ "Group Dashboard" }</h1>

            {
                if let Some(rs) = (*round_status).clone() {
                    html!{
                        <div class="info-card" style="margin-bottom:10px;">
                            <p><strong>{ "Round:" }</strong>{ format!(" {}", rs.round_number) }</p>
                            <p><strong>{ "Status:" }</strong>{ if rs.round_open { " OPEN" } else { " CLOSED" } }</p>
                        </div>
                    }
                } else { html!{} }
            }

            {
                if session_id_str.trim().is_empty() {
                    html!{
                        <div class="error-message">
                            <p>{ "No session ID found. Please log in again." }</p>
                            <Link<Route> to={Route::Login}>{ "Go to Login" }</Link<Route>>
                        </div>
                    }
                } else if let Some(g) = (*group).clone() {
                    html!{
                        <>
                            <div class="info-card">
                                <p><strong>{ "Name:" }</strong> { &g.name }</p>
                                <p><strong>{ "Email:" }</strong> { &g.email }</p>
                            </div>

                            <div class="preferences-section">
                                <h3>{ "Final match (approved)" }</h3>

                                {
                                    if !(*final_error).is_empty() {
                                        html! { <div class="error-message">{ (*final_error).clone() }</div> }
                                    } else { html!{} }
                                }

                                <button class="btn btn-primary" onclick={{
                                    let r = refresh_final_matches.clone();
                                    Callback::from(move |_| r.emit(()))
                                }}>{ "Refresh finals" }</button>

                                {
                                    if (*final_matches).is_empty() {
                                        html!{ <p><i>{ "No final match yet." }</i></p> }
                                    } else {
                                        html!{
                                            <ul class="preferences-list">
                                                { for (*final_matches).iter().map(|m| {
                                                    let status = m.status.clone().unwrap_or_else(|| "final".into());
                                                    let company_label = company_name_by_email(&m.company_email);

                                                    html!{
                                                        <li key={format!("final:{}:{}", m.project_id, m.company_email)}>
                                                            <div style="display:flex; flex-direction:column; gap:4px;">
                                                                <span><strong>{ "Company: " }</strong>{ company_label }</span>
                                                                <span><strong>{ "Project: " }</strong>{ m.project_name.clone() }</span>
                                                                <span><strong>{ "Status: " }</strong>{ status }</span>
                                                            </div>
                                                        </li>
                                                    }
                                                }) }
                                            </ul>
                                        }
                                    }
                                }
                            </div>

                            <div class="preferences-section">
                                <h3>{ "Current matches (this round)" }</h3>

                                {
                                    if !(*match_error).is_empty() {
                                        html! { <div class="error-message">{ (*match_error).clone() }</div> }
                                    } else { html!{} }
                                }
                                {
                                    if !(*match_msg).is_empty() {
                                        html! { <div class="summary">{ (*match_msg).clone() }</div> }
                                    } else { html!{} }
                                }

                                <div class="controls" style="margin-top:10px; margin-bottom:10px;">
                                    <button class="btn btn-primary" onclick={{
                                        let r = refresh_current_matches.clone();
                                        Callback::from(move |_| r.emit(()))
                                    }}>{ "Refresh current" }</button>
                                </div>

                                {
                                    if (*current_matches).is_empty() {
                                        html!{ <p><i>{ "No match for you right now." }</i></p> }
                                    } else {
                                        html!{
                                            <ul class="preferences-list">
                                                { for (*current_matches).iter().map(|m| {
                                                    let status = m.status.clone().unwrap_or_else(|| "pending".into());
                                                    let company_label = company_name_by_email(&m.company_email);

                                                    html!{
                                                        <li key={format!("{}::{}", m.project_id, m.company_email)}>
                                                            <div style="display:flex; flex-direction:column; gap:4px;">
                                                                <span><strong>{ "Company: " }</strong>{ company_label }</span>
                                                                <span><strong>{ "Project: " }</strong>{ m.project_name.clone() }</span>
                                                                <span><strong>{ "Status: " }</strong>{ status }</span>

                                                                <div class="controls" style="margin-top:8px;">
                                                                    <button class="btn btn-success" onclick={accept_match.clone()}>{ "Accept" }</button>
                                                                    <button class="btn btn-danger" onclick={reject_match.clone()} style="margin-left:8px;">{ "Reject" }</button>
                                                                </div>
                                                            </div>
                                                        </li>
                                                    }
                                                }) }
                                            </ul>
                                        }
                                    }
                                }
                            </div>

                            <div class="preferences-section">
                                <h3>{ "Your ranked projects" }</h3>

                                {
                                    if !(*error).is_empty() {
                                        html!{ <div class="error-message">{ (*error).clone() }</div> }
                                    } else { html!{} }
                                }

                                {
                                    if ranked_projects.is_empty() {
                                        html!{ <p><i>{ "No projects ranked yet. Add projects below." }</i></p> }
                                    } else {
                                        html!{
                                            <ul class="preferences-list">
                                                { for ranked_projects.iter().enumerate().map(|(idx, p)| {
                                                    let pid = p.id.clone();
                                                    let company_label = company_name_by_email(&p.company_email);
                                                    let desc = p.description.clone().unwrap_or_else(|| "No description".into());

                                                    html!{
                                                        <li key={pid.clone()}>
                                                            <div style="display:flex; flex-direction:column;">
                                                                <span>
                                                                    <strong>{ format!("{}. {}", idx + 1, p.name) }</strong>
                                                                    <span class="already-added">{ format!("{}", company_label) }</span>
                                                                </span>
                                                                <span style="opacity:0.8;">{ desc }</span>
                                                            </div>

                                                            <button
                                                                class="btn btn-danger"
                                                                style="margin-left:10px;"
                                                                onclick={{
                                                                    let rm = on_remove_project.clone();
                                                                    Callback::from(move |_| rm.emit(pid.clone()))
                                                                }}
                                                            >
                                                                { "Remove" }
                                                            </button>
                                                        </li>
                                                    }
                                                }) }
                                            </ul>
                                        }
                                    }
                                }
                            </div>

                            <div class="available-list">
                                <h3>{ "Available projects" }</h3>

                                {
                                    if available_projects.is_empty() {
                                        html!{ <p><i>{ "No active projects available" }</i></p> }
                                    } else {
                                        html!{
                                            <ul class="list-items">
                                                { for available_projects.iter().map(|p| {
                                                    let already = (*prefs).iter().any(|x| x == &p.id);
                                                    let pid = p.id.clone();
                                                    let company_label = company_name_by_email(&p.company_email);

                                                    html!{
                                                        <li key={pid.clone()} style="display:flex; flex-direction:column; align-items:flex-start; gap:8px;">
                                                            <div style="width:100%;">
                                                                <strong>{ &p.name }</strong>
                                                                <span class="already-added">{ format!("{}", company_label) }</span>
                                                            </div>

                                                            {
                                                                if let Some(d) = &p.description {
                                                                    html!{ <div style="opacity:0.85;">{ d.clone() }</div> }
                                                                } else {
                                                                    html!{ <div style="opacity:0.6; font-style:italic;">{ "No description" }</div> }
                                                                }
                                                            }

                                                            <div style="display:flex; gap:10px; align-items:center;">
                                                                <span class="already-added">{ format!("capacity: {}", p.capacity) }</span>
                                                                {
                                                                    if already {
                                                                        html!{ <span class="already-added">{ "(already ranked)" }</span> }
                                                                    } else {
                                                                        html!{
                                                                            <button
                                                                                class="btn btn-primary"
                                                                                onclick={{
                                                                                    let add = on_add_project.clone();
                                                                                    Callback::from(move |_| add.emit(pid.clone()))
                                                                                }}
                                                                            >
                                                                                { "Add" }
                                                                            </button>
                                                                        }
                                                                    }
                                                                }
                                                            </div>
                                                        </li>
                                                    }
                                                }) }
                                            </ul>
                                        }
                                    }
                                }
                            </div>

                            <div class="navigation">
                                <Link<Route> to={Route::MatchPage} classes="btn btn-primary">
                                    { "See Matches" }
                                </Link<Route>>
                            </div>
                        </>
                    }
                } else {
                    html!{ <div class="loading"><p>{ "Loading..." }</p></div> }
                }
            }
        </div>
    }
}
