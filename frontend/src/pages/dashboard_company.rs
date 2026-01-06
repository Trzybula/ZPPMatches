use yew::prelude::*;
use yew_router::prelude::Link;
use crate::Route;
use gloo_net::http::Request;
use gloo_storage::{LocalStorage, Storage};
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlInputElement, HtmlSelectElement};

use shared::{
    Company, Group, Project,
    ProjectPreferencesResponse,
    CreateProjectRequest, SetProjectPreferencesRequest,
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

fn get_session_id() -> String {
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

#[function_component(DashboardCompanyPage)]
pub fn dashboard_company_page() -> Html {
    let company = use_state(|| None::<Company>);
    let groups = use_state(|| Vec::<Group>::new());
    let projects = use_state(|| Vec::<Project>::new());

    let selected_project_id = use_state(|| "".to_string());
    let prefs = use_state(|| Vec::<String>::new());
    let error = use_state(|| "".to_string());

    let new_project_name = use_state(|| "".to_string());
    let new_project_desc = use_state(|| "".to_string());
    let project_error = use_state(|| "".to_string());
    let current_matches = use_state(|| Vec::<CurrentMatchResponse>::new());
    let final_matches = use_state(|| Vec::<CurrentMatchResponse>::new());

    let match_error = use_state(|| "".to_string());
    let match_msg = use_state(|| "".to_string());
    let final_error = use_state(|| "".to_string());
    let round_status = use_state(|| None::<RoundStatusResponse>);

    let session_id = use_state(get_session_id);
    let session_str = (*session_id).clone();

    {
        let selected_project_id = selected_project_id.clone();
        let prefs = prefs.clone();
        let error = error.clone();

        use_effect_with((), move |_| {
            selected_project_id.set("".into());
            prefs.set(vec![]);
            error.set("".into());
            || ()
        });
    }
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
        let company = company.clone();
        let session_id = session_id.clone();

        use_effect_with((), move |_| {
            spawn_local(async move {
                let session = clean_session_id((*session_id).clone());
                if session.trim().is_empty() {
                    company.set(None);
                    return;
                }

                let url = format!("{}?session_id={}", api("/company/me"), session);
                if let Ok(resp) = Request::get(&url).send().await {
                    if let Ok(data) = resp.json::<Option<Company>>().await {
                        company.set(data);
                    }
                }
            });
            || ()
        });
    }

    {
        let groups = groups.clone();
        use_effect_with((), move |_| {
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api("/group/list")).send().await {
                    if let Ok(list) = resp.json::<Vec<Group>>().await {
                        groups.set(list);
                    }
                }
            });
            || ()
        });
    }
    let refresh_projects: Callback<()> = {
        let projects = projects.clone();
        Callback::from(move |_| {
            let projects = projects.clone();
            spawn_local(async move {
                if let Ok(resp) = Request::get(&api("/projects")).send().await {
                    if let Ok(list) = resp.json::<Vec<Project>>().await {
                        projects.set(list);
                    }
                }
            });
        })
    };

    {
        let refresh_projects = refresh_projects.clone();
        use_effect_with((), move |_| {
            refresh_projects.emit(());
            || ()
        });
    }
    {
        let company = company.clone();
        let projects = projects.clone();
        let selected_project_id = selected_project_id.clone();
        let prefs = prefs.clone();
        let error = error.clone();

        let company_email_dep: Option<String> = (*company).as_ref().map(|c| c.email.clone());
        let projects_len_dep: usize = (*projects).len();

        use_effect_with((company_email_dep, projects_len_dep), move |_| {
            let maybe_company = (*company).clone();
            let all_projects = (*projects).clone();
            let selected = (*selected_project_id).clone();

            if let Some(c) = maybe_company {
                let my_ids: Vec<String> = all_projects
                    .iter()
                    .filter(|p| p.company_email == c.email)
                    .map(|p| p.id.clone())
                    .collect();

                let selected_ok =
                    !selected.trim().is_empty() && my_ids.iter().any(|id| id == &selected);

                if !selected_ok {
                    selected_project_id.set("".into());
                    prefs.set(vec![]);
                    error.set("".into());
                }
            } else {
                selected_project_id.set("".into());
                prefs.set(vec![]);
                error.set("".into());
            }

            || ()
        });
    }
    {
        let prefs = prefs.clone();
        let selected_project_id = selected_project_id.clone();

        use_effect_with((*selected_project_id).clone(), move |pid| {
            let pid = pid.clone();
            let prefs = prefs.clone();

            spawn_local(async move {
                if pid.trim().is_empty() {
                    prefs.set(vec![]);
                    return;
                }

                let url = format!("{}?project_id={}", api("/project/preferences"), pid);
                if let Ok(resp) = Request::get(&url).send().await {
                    if let Ok(data) = resp.json::<Option<ProjectPreferencesResponse>>().await {
                        prefs.set(data.map(|d| d.group_emails_ranked).unwrap_or_default());
                    } else {
                        prefs.set(vec![]);
                    }
                } else {
                    prefs.set(vec![]);
                }
            });

            || ()
        });
    }

    let save_prefs = {
        let session_id = session_id.clone();
        let error = error.clone();

        Callback::from(move |(project_id, next): (String, Vec<String>)| {
            let session_id = session_id.clone();
            let error = error.clone();

            spawn_local(async move {
                let session = clean_session_id((*session_id).clone());
                if session.trim().is_empty() || project_id.trim().is_empty() {
                    return;
                }

                let req = SetProjectPreferencesRequest {
                    session_id: session,
                    project_id,
                    group_emails_ranked: next,
                };

                let resp = Request::post(&api("/project/preferences"))
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&req).unwrap())
                    .unwrap()
                    .send()
                    .await;

                match resp {
                    Ok(r) => match r.json::<bool>().await {
                        Ok(true) => error.set("".into()),
                        Ok(false) => error.set("Could not save ranking.".into()),
                        Err(_) => {
                            if r.status() != 200 {
                                error.set(format!(" Save failed: HTTP {}", r.status()));
                            }
                        }
                    },
                    Err(e) => error.set(format!(" Request error: {:?}", e)),
                }
            });
        })
    };

    let on_create_project: Callback<MouseEvent> = {
        let new_project_name = new_project_name.clone();
        let new_project_desc = new_project_desc.clone();
        let project_error = project_error.clone();
        let session_id = session_id.clone();
        let refresh_projects = refresh_projects.clone();
        let selected_project_id = selected_project_id.clone();

        Callback::from(move |_| {
            let name = (*new_project_name).trim().to_string();
            let desc = (*new_project_desc).trim().to_string();

            if name.is_empty() {
                project_error.set("Project name required".into());
                return;
            }

            new_project_name.set("".into());
            new_project_desc.set("".into());
            project_error.set("".into());

            let session_id2 = session_id.clone();
            let refresh_projects2 = refresh_projects.clone();
            let selected_project_id2 = selected_project_id.clone();
            let project_error2 = project_error.clone();

            spawn_local(async move {
                let session = clean_session_id((*session_id2).clone());
                if session.trim().is_empty() {
                    project_error2.set("Please log in again.".into());
                    return;
                }

                let payload = CreateProjectRequest {
                    id: "".into(),
                    company_email: "".into(),
                    name,
                    description: if desc.is_empty() { None } else { Some(desc) },
                    capacity: 1,
                    active: true,
                };

                let url = format!("{}?session_id={}", api("/company/projects"), session);

                match Request::post(&url)
                    .header("Content-Type", "application/json")
                    .body(serde_json::to_string(&payload).unwrap())
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) => {
                        let ok = resp.json::<bool>().await.unwrap_or(false);
                        if ok {
                            refresh_projects2.emit(());
                            selected_project_id2.set("".into());
                        } else {
                            project_error2.set("Could not create project".into());
                        }
                    }
                    Err(e) => project_error2.set(format!("Could not create project: {:?}", e)),
                }
            });
        })
    };

    let on_project_select = {
        let selected_project_id = selected_project_id.clone();
        let error = error.clone();
        Callback::from(move |e: Event| {
            let val = e.target_unchecked_into::<HtmlSelectElement>().value();
            selected_project_id.set(val);
            error.set("".into());
        })
    };

    let on_add_group = {
        let prefs = prefs.clone();
        let selected_project_id = selected_project_id.clone();
        let error = error.clone();
        let save_prefs = save_prefs.clone();

        Callback::from(move |email: String| {
            if (*selected_project_id).trim().is_empty() {
                error.set("Choose project first".into());
                return;
            }
            if (*prefs).contains(&email) {
                error.set("This group is already in ranking.".into());
                return;
            }

            let mut next = (*prefs).clone();
            next.push(email.clone());
            prefs.set(next.clone());
            error.set("".into());

            save_prefs.emit(((*selected_project_id).clone(), next));
        })
    };

    let on_remove_group = {
        let prefs = prefs.clone();
        let selected_project_id = selected_project_id.clone();
        let save_prefs = save_prefs.clone();

        Callback::from(move |email: String| {
            if (*selected_project_id).trim().is_empty() {
                return;
            }
            let next: Vec<_> = (*prefs).iter().cloned().filter(|e| e != &email).collect();
            prefs.set(next.clone());
            save_prefs.emit(((*selected_project_id).clone(), next));
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
                let session = clean_session_id((*session_id).clone());
                if session.trim().is_empty() {
                    current_matches.set(vec![]);
                    return;
                }

                match_error.set("".into());
                match_msg.set("".into());

                let url = format!("{}?session_id={}", api("/me/match"), session);
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
                let session = clean_session_id((*session_id).clone());
                if session.trim().is_empty() {
                    final_matches.set(vec![]);
                    return;
                }

                final_error.set("".into());

                let url = format!("{}?session_id={}", api("/me/final"), session);
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

        Callback::from(move |project_id: String| {
            let session_id = session_id.clone();
            let match_error = match_error.clone();
            let match_msg = match_msg.clone();
            let refresh_current_matches = refresh_current_matches.clone();
            let refresh_final_matches = refresh_final_matches.clone();

            spawn_local(async move {
                let session = clean_session_id((*session_id).clone());
                if session.trim().is_empty() {
                    return;
                }

                match_error.set("".into());
                match_msg.set("".into());

                let body = MatchActionRequest {
                    session_id: session,
                    project_id: Some(project_id),
                };

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

        Callback::from(move |project_id: String| {
            let session_id = session_id.clone();
            let match_error = match_error.clone();
            let match_msg = match_msg.clone();
            let refresh_current_matches = refresh_current_matches.clone();
            let refresh_final_matches = refresh_final_matches.clone();

            spawn_local(async move {
                let session = clean_session_id((*session_id).clone());
                if session.trim().is_empty() {
                    return;
                }

                match_error.set("".into());
                match_msg.set("".into());

                let body = MatchActionRequest {
                    session_id: session,
                    project_id: Some(project_id),
                };

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

    html! {
        <div class="dashboard-common dashboard-company">
            <h1>{ "Company Dashboard" }</h1>

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
                if session_str.trim().is_empty() {
                    html!{
                        <div class="error-message">
                            <p>{ "Please log in again." }</p>
                            <Link<Route> to={Route::Login}>{ "Go to Login" }</Link<Route>>
                        </div>
                    }
                } else if let Some(c) = (*company).clone() {
                    let my_projects: Vec<Project> = (*projects)
                        .iter()
                        .filter(|p| p.company_email == c.email)
                        .cloned()
                        .collect();

                    let project_selected = !(*selected_project_id).trim().is_empty();

                    html!{
                        <>
                            <div class="info-card">
                                <p><strong>{ "Name:" }</strong> { &c.name }</p>
                                <p><strong>{ "Email:" }</strong> { &c.email }</p>
                            </div>

                            <div class="preferences-section">
                                <h3>{ "Final matches (approved)" }</h3>

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
                                        html!{ <p><i>{ "No final matches yet." }</i></p> }
                                    } else {
                                        html!{
                                            <ul class="list-items">
                                                { for (*final_matches).iter().map(|m| html!{
                                                    <li key={format!("final:{}:{}", m.project_id, m.group_email)}>
                                                        <div style="display:flex; flex-direction:column; gap:4px;">
                                                            <div><strong>{ "Project:" }</strong>{ format!(" {}", m.project_name) }</div>
                                                            <div><strong>{ "Group:" }</strong>{ format!(" {}", m.group_email) }</div>
                                                            <div><strong>{ "Status:" }</strong>{ " final" }</div>
                                                        </div>
                                                    </li>
                                                }) }
                                            </ul>
                                        }
                                    }
                                }
                            </div>

                            <div class="preferences-section">
                                <h3>{ "New matches (not approved yet)" }</h3>

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

                                <button class="btn btn-primary" onclick={{
                                    let r = refresh_current_matches.clone();
                                    Callback::from(move |_| r.emit(()))
                                }}>{ "Refresh current" }</button>

                                {
                                    if (*current_matches).is_empty() {
                                        html!{ <p><i>{ "No matches for your projects right now." }</i></p> }
                                    } else {
                                        html!{
                                            <ul class="list-items">
                                                { for (*current_matches).iter().map(|m| {
                                                    let status = m.status.clone().unwrap_or_else(|| "pending".into());
                                                    let pid = m.project_id.clone();

                                                    html!{
                                                        <li key={format!("{}-{}", m.project_id, m.group_email)}
                                                        style="display:flex; flex-direction:column; gap:12px;"
                                                    >
                                                            <div style="display:flex; flex-direction:column; gap:4px;">
                                                                <div><strong>{ "Project:" }</strong>{ format!(" {}", m.project_name) }</div>
                                                                <div><strong>{ "Group:" }</strong>{ format!(" {}", m.group_email) }</div>
                                                                <div><strong>{ "Status:" }</strong>{ format!(" {}", status) }</div>
                                                            </div>

                                                            <div class="controls" style="margin-top:12px; display:flex; gap:10px;">
                                                                <button class="btn btn-success" onclick={{
                                                                    let a = accept_match.clone();
                                                                    let pid2 = pid.clone();
                                                                    Callback::from(move |_| a.emit(pid2.clone()))
                                                                }}>{ "Accept" }</button>

                                                                <button class="btn btn-danger" onclick={{
                                                                    let rj = reject_match.clone();
                                                                    let pid2 = pid.clone();
                                                                    Callback::from(move |_| rj.emit(pid2.clone()))
                                                                }}>{ "Reject" }</button>
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
                                <h3>{ "Create Project" }</h3>

                                <div class="input-group">
                                    <input
                                        placeholder="Project name"
                                        value={(*new_project_name).clone()}
                                        oninput={{
                                            let s = new_project_name.clone();
                                            let pe = project_error.clone();
                                            Callback::from(move |e: InputEvent| {
                                                s.set(e.target_unchecked_into::<HtmlInputElement>().value());
                                                if !(*pe).is_empty() { pe.set("".into()); }
                                            })
                                        }}
                                    />
                                </div>

                                <div class="input-group">
                                    <input
                                        placeholder="Description (optional)"
                                        value={(*new_project_desc).clone()}
                                        oninput={{
                                            let s = new_project_desc.clone();
                                            Callback::from(move |e: InputEvent| {
                                                s.set(e.target_unchecked_into::<HtmlInputElement>().value());
                                            })
                                        }}
                                    />
                                </div>

                                <button class="btn btn-success" onclick={on_create_project}>{ "Create" }</button>

                                {
                                    if !(*project_error).is_empty() {
                                        html! { <div class="error-message">{ (*project_error).clone() }</div> }
                                    } else { html!{} }
                                }
                            </div>

                            <div class="preferences-section">
                                <h3>{ "Select Project" }</h3>

                                <select value={(*selected_project_id).clone()} onchange={on_project_select}>
                                    <option value="">{ "-- choose project --" }</option>
                                    { for my_projects.iter().map(|p| html!{
                                        <option value={p.id.clone()}>{ &p.name }</option>
                                    }) }
                                </select>

                                {
                                    if my_projects.is_empty() {
                                        html! { <p><i>{ "You have no projects yet. Create one above." }</i></p> }
                                    } else { html!{} }
                                }
                            </div>

                            <div class="preferences-section">
                                <h3>{ "Ranking" }</h3>

                                {
                                    if !(*error).is_empty() {
                                        html! { <div class="error-message">{ (*error).clone() }</div> }
                                    } else { html!{} }
                                }

                                {
                                    if !project_selected {
                                        html! { <p><i>{ "Choose project first." }</i></p> }
                                    } else if (*prefs).is_empty() {
                                        html! { <p><i>{ "No groups ranked yet." }</i></p> }
                                    } else {
                                        html! {
                                            <ol class="preferences-list">
                                                { for (*prefs).iter().map(|email| html!{
                                                    <li key={email.clone()}>
                                                        <span>{ email }</span>
                                                        <button
                                                            class="btn btn-danger"
                                                            style="margin-left:10px;"
                                                            onclick={{
                                                                let rm = on_remove_group.clone();
                                                                let e = email.clone();
                                                                Callback::from(move |_| rm.emit(e.clone()))
                                                            }}
                                                        >
                                                            { "Remove" }
                                                        </button>
                                                    </li>
                                                }) }
                                            </ol>
                                        }
                                    }
                                }
                            </div>

                            <div class="available-list">
                                <h3>{ "Available Groups" }</h3>

                                {
                                    if (*groups).is_empty() {
                                        html! { <p><i>{ "No groups available" }</i></p> }
                                    } else {
                                        html! {
                                            <ul class="list-items">
                                                { for (*groups).iter().map(|g| {
                                                    let already = (*prefs).contains(&g.email);

                                                    html!{
                                                        <li key={g.email.clone()}>
                                                            <span>{ &g.name }</span>

                                                            {
                                                                if already {
                                                                    html!{ <span class="already-added">{ " (added)" }</span> }
                                                                } else {
                                                                    html!{
                                                                        <button
                                                                            class="btn btn-primary"
                                                                            disabled={!project_selected}
                                                                            onclick={{
                                                                                let add = on_add_group.clone();
                                                                                let e = g.email.clone();
                                                                                Callback::from(move |_| add.emit(e.clone()))
                                                                            }}
                                                                        >
                                                                            { "Add" }
                                                                        </button>
                                                                    }
                                                                }
                                                            }
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
