use axum::{
    Json,
    extract::{Query, State}
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex}
};
use uuid::Uuid;

use crate::{
    models::{Group, Company, LoginRequest, LoginResponse, MatchResult, AddPref},
    state::AppState,
    matching::stable_matching,
};

pub async fn add_group(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(group): Json<Group>
) -> Json<LoginResponse> {
    let mut s = state.lock().unwrap();
    if s.groups.iter().any(|g| g.email == group.email) {
        return Json(LoginResponse {
            ok: false,
            message: "Group with this email already exists".into(),
            session_id: None,
            email: None,
            role: None,
        });
    }

    s.groups.push(group);
    let _ = s.save();

    Json(LoginResponse {
        ok: true,
        message: "Group created successfully".into(),
        session_id: None,
        email: None,
        role: None,
    })
}

pub async fn add_company(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(company): Json<Company>
) -> Json<LoginResponse> {
    let mut s = state.lock().unwrap();
    if s.companies.iter().any(|c| c.email == company.email) {
        return Json(LoginResponse {
            ok: false,
            message: "Company with this email already exists".into(),
            session_id: None,
            email: None,
            role: None,
        });
    }

    s.companies.push(company);
    let _ = s.save();

    Json(LoginResponse {
        ok: true,
        message: "Company created successfully".into(),
        session_id: None,
        email: None,
        role: None,
    })
}

pub async fn match_groups(
    State(state): State<Arc<Mutex<AppState>>>
) -> Json<Vec<MatchResult>> {
    let s = state.lock().unwrap();
    Json(stable_matching(&s.groups, &s.companies))
}

pub async fn login_group(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(login): Json<LoginRequest>
) -> Json<LoginResponse> {
    let mut s = state.lock().unwrap();

    let found = s.groups.iter().find(|g|
        g.email == login.email && g.password == login.password
    );

    if let Some(_) = found {
        let session_id = Uuid::new_v4().to_string();
        s.sessions.insert(session_id.clone(), login.email.clone());
        let _ = s.save();

        Json(LoginResponse {
            ok: true,
            message: "Group login success".into(),
            session_id: Some(session_id),
            email: Some(login.email),
            role: Some("group".into()),
        })
    } else {
        Json(LoginResponse {
            ok: false,
            message: "Invalid credentials".into(),
            session_id: None,
            email: None,
            role: None,
        })
    }
}

pub async fn login_company(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(login): Json<LoginRequest>
) -> Json<LoginResponse> {
    let mut s = state.lock().unwrap();

    let found = s.companies.iter().find(|c|
        c.email == login.email && c.password == login.password
    );

    if let Some(_) = found {
        let session_id = Uuid::new_v4().to_string();
        s.sessions.insert(session_id.clone(), login.email.clone());
        let _ = s.save();

        Json(LoginResponse {
            ok: true,
            message: "Company login success".into(),
            session_id: Some(session_id),
            email: Some(login.email),
            role: Some("company".into()),
        })
    } else {
        Json(LoginResponse {
            ok: false,
            message: "Invalid credentials".into(),
            session_id: None,
            email: None,
            role: None,
        })
    }
}

pub async fn group_me(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Option<Group>> {
    let session = params.get("session_id").cloned().unwrap_or_default();

    let s = state.lock().unwrap();

    let email = match s.sessions.get(&session) {
        Some(e) => e.clone(),
        None => return Json(None),
    };

    let g = s.groups.iter().find(|g| g.email == email).cloned();
    Json(g)
}

pub async fn company_me(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Option<Company>> {
    let session = params.get("session_id").cloned().unwrap_or_default();

    let s = state.lock().unwrap();

    let email = match s.sessions.get(&session) {
        Some(e) => e.clone(),
        None => return Json(None),
    };

    let c = s.companies.iter().find(|c| c.email == email).cloned();
    Json(c)
}

pub async fn list_companies(
    State(state): State<Arc<Mutex<AppState>>>
) -> Json<Vec<Company>> {
    let s = state.lock().unwrap();
    Json(s.companies.clone())
}

pub async fn list_groups(
    State(state): State<Arc<Mutex<AppState>>>
) -> Json<Vec<Group>> {
    let s = state.lock().unwrap();
    Json(s.groups.clone())
}

pub async fn group_add_pref(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(body): Json<AddPref>
) -> Json<bool> {
    let mut s = state.lock().unwrap();

    let email = match s.sessions.get(&body.session_id) {
        Some(e) => e.clone(),
        None => return Json(false),
    };

    if let Some(g) = s.groups.iter_mut().find(|x| x.email == email) {
        if !g.preferences.contains(&body.pref) {
            g.preferences.push(body.pref.clone());
            
            let _ = s.save();
            
            return Json(true);
        }
        return Json(true);
    }

    Json(false)
}

pub async fn company_add_pref(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(body): Json<AddPref>
) -> Json<bool> {
    let mut s = state.lock().unwrap();

    let email = match s.sessions.get(&body.session_id) {
        Some(e) => e.clone(),
        None => return Json(false),
    };

    if let Some(c) = s.companies.iter_mut().find(|x| x.email == email) {
        if !c.preferences.contains(&body.pref) {
            c.preferences.push(body.pref.clone());
            let _ = s.save();
            
            return Json(true);
        }
        return Json(true);
    }

    Json(false)
}