use axum::{
    Json,
    extract::{Query, State},
    http::StatusCode,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};
use uuid::Uuid;

use shared::{
    CreateCompanyRequest, CreateGroupRequest, CurrentMatchResponse, LoginRequest, LoginResponse,
    MatchActionRequest, RoundStatusResponse, SetGroupPreferencesRequest,
    SetProjectPreferencesRequest,
};

use crate::{
    matching::stable_matching,
    models::{
        AuthUser, Company, Group, GroupPreferences, MatchDecision, MatchResult, Project,
        ProjectPreferences, Role,
    },
    state::{AppState, Session},
};

fn get_session(s: &AppState, session_id: &str) -> Option<Session> {
    s.sessions.get(session_id).cloned()
}

fn role_to_str(role: &Role) -> &'static str {
    match role {
        Role::Admin => "admin",
        Role::Company => "company",
        Role::Group => "group",
    }
}

fn is_admin(s: &AppState, session_id: &str) -> bool {
    s.sessions
        .get(session_id)
        .map(|sess| sess.role == Role::Admin)
        .unwrap_or(false)
}

#[allow(dead_code)]
fn is_group(s: &AppState, session_id: &str) -> bool {
    s.sessions
        .get(session_id)
        .map(|sess| sess.role == Role::Group)
        .unwrap_or(false)
}

#[allow(dead_code)]
fn is_company(s: &AppState, session_id: &str) -> bool {
    s.sessions
        .get(session_id)
        .map(|sess| sess.role == Role::Company)
        .unwrap_or(false)
}

pub async fn add_group(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(req): Json<CreateGroupRequest>,
) -> Json<LoginResponse> {
    let mut s = state.lock().unwrap();

    if s.users.iter().any(|u| u.email == req.email) {
        return Json(LoginResponse {
            ok: false,
            message: "User with this email already exists".into(),
            session_id: None,
            email: None,
            role: None,
        });
    }

    s.users.push(AuthUser {
        email: req.email.clone(),
        password: req.password,
        role: Role::Group,
    });

    s.groups.push(Group {
        name: req.name,
        email: req.email,
    });

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
    Json(req): Json<CreateCompanyRequest>,
) -> Json<LoginResponse> {
    let mut s = state.lock().unwrap();

    if s.users.iter().any(|u| u.email == req.email) {
        return Json(LoginResponse {
            ok: false,
            message: "User with this email already exists".into(),
            session_id: None,
            email: None,
            role: None,
        });
    }

    s.users.push(AuthUser {
        email: req.email.clone(),
        password: req.password,
        role: Role::Company,
    });

    s.companies.push(Company {
        name: req.name,
        email: req.email,
    });

    let _ = s.save();

    Json(LoginResponse {
        ok: true,
        message: "Company created successfully".into(),
        session_id: None,
        email: None,
        role: None,
    })
}

pub async fn login(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(login): Json<LoginRequest>,
) -> Json<LoginResponse> {
    let mut s = state.lock().unwrap();

    let found = s
        .users
        .iter()
        .find(|u| u.email == login.email && u.password == login.password)
        .map(|u| (u.email.clone(), u.role.clone()));

    match found {
        Some((email, role)) => {
            let session_id = Uuid::new_v4().to_string();

            s.sessions.insert(
                session_id.clone(),
                Session {
                    email: email.clone(),
                    role: role.clone(),
                },
            );

            let _ = s.save();

            Json(LoginResponse {
                ok: true,
                message: "Login success".into(),
                session_id: Some(session_id),
                email: Some(email),
                role: Some(role_to_str(&role).to_string()),
            })
        }
        None => Json(LoginResponse {
            ok: false,
            message: "Invalid credentials".into(),
            session_id: None,
            email: None,
            role: None,
        }),
    }
}

pub async fn group_me(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Option<Group>> {
    let session = params.get("session_id").cloned().unwrap_or_default();

    let s = state.lock().unwrap();

    let sess = match get_session(&s, &session) {
        Some(x) => x,
        None => return Json(None),
    };

    if sess.role != Role::Group {
        return Json(None);
    }

    Json(s.groups.iter().find(|g| g.email == sess.email).cloned())
}

pub async fn company_me(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Option<Company>> {
    let session = params.get("session_id").cloned().unwrap_or_default();

    let s = state.lock().unwrap();

    let sess = match get_session(&s, &session) {
        Some(x) => x,
        None => return Json(None),
    };

    if sess.role != Role::Company {
        return Json(None);
    }

    Json(s.companies.iter().find(|c| c.email == sess.email).cloned())
}

pub async fn list_groups(State(state): State<Arc<Mutex<AppState>>>) -> Json<Vec<Group>> {
    let s = state.lock().unwrap();
    Json(s.groups.clone())
}

pub async fn list_companies(State(state): State<Arc<Mutex<AppState>>>) -> Json<Vec<Company>> {
    let s = state.lock().unwrap();
    Json(s.companies.clone())
}

pub async fn list_projects(State(state): State<Arc<Mutex<AppState>>>) -> Json<Vec<Project>> {
    let s = state.lock().unwrap();
    Json(s.projects.clone())
}

pub async fn company_add_project(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
    Json(mut project): Json<Project>,
) -> Json<bool> {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    let mut s = state.lock().unwrap();

    let sess = match get_session(&s, &session_id) {
        Some(x) => x,
        None => return Json(false),
    };

    if sess.role != Role::Company {
        return Json(false);
    }

    project.company_email = sess.email.clone();

    if project.id.trim().is_empty() {
        project.id = Uuid::new_v4().to_string();
    }
    if project.capacity == 0 {
        project.capacity = 1;
    }

    s.projects.push(project);
    let _ = s.save();

    Json(true)
}

pub async fn set_group_preferences(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(body): Json<SetGroupPreferencesRequest>,
) -> Json<bool> {
    let mut s = state.lock().unwrap();

    let sess = match get_session(&s, &body.session_id) {
        Some(x) => x,
        None => return Json(false),
    };

    if sess.role != Role::Group {
        return Json(false);
    }

    if let Some(p) = s
        .group_prefs
        .iter_mut()
        .find(|p| p.group_email == sess.email)
    {
        p.project_ids_ranked = body.project_ids_ranked;
    } else {
        s.group_prefs.push(GroupPreferences {
            group_email: sess.email,
            project_ids_ranked: body.project_ids_ranked,
        });
    }

    let _ = s.save();
    Json(true)
}

pub async fn set_project_preferences(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(body): Json<SetProjectPreferencesRequest>,
) -> Json<bool> {
    let mut s = state.lock().unwrap();

    let sess = match get_session(&s, &body.session_id) {
        Some(x) => x,
        None => return Json(false),
    };

    if sess.role != Role::Company {
        return Json(false);
    }

    let owns = s
        .projects
        .iter()
        .any(|p| p.id == body.project_id && p.company_email == sess.email);

    if !owns {
        return Json(false);
    }

    if let Some(p) = s
        .project_prefs
        .iter_mut()
        .find(|p| p.project_id == body.project_id)
    {
        p.group_emails_ranked = body.group_emails_ranked;
    } else {
        s.project_prefs.push(ProjectPreferences {
            project_id: body.project_id,
            group_emails_ranked: body.group_emails_ranked,
        });
    }

    let _ = s.save();
    Json(true)
}

pub async fn match_groups(State(state): State<Arc<Mutex<AppState>>>) -> Json<Vec<MatchResult>> {
    let (groups, projects, group_prefs, project_prefs) = {
        let s = state.lock().unwrap();
        (
            s.groups.clone(),
            s.projects.clone(),
            s.group_prefs.clone(),
            s.project_prefs.clone(),
        )
    };

    let results = stable_matching(&groups, &projects, &group_prefs, &project_prefs);

    Json(results)
}

fn compute_matches_filtered(s: &AppState) -> Vec<MatchResult> {
    stable_matching(&s.groups, &s.projects, &s.group_prefs, &s.project_prefs)
        .into_iter()
        .filter(|m| {
            if s.is_rejected(&m.group_email, &m.project_id) {
                return false;
            }
            if s.is_group_accepted(&m.group_email) {
                return false;
            }
            if s.is_project_full_or_accepted(&m.project_id) {
                return false;
            }
            true
        })
        .collect()
}

pub async fn get_group_preferences(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Option<GroupPreferences>> {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    let s = state.lock().unwrap();

    let sess = match s.sessions.get(&session_id) {
        Some(x) => x.clone(),
        None => return Json(None),
    };

    if sess.role != Role::Group {
        return Json(None);
    }

    Json(
        s.group_prefs
            .iter()
            .find(|p| p.group_email == sess.email)
            .cloned(),
    )
}

pub async fn get_project_preferences(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> Json<Option<ProjectPreferences>> {
    let project_id = params.get("project_id").cloned().unwrap_or_default();
    let s = state.lock().unwrap();

    Json(
        s.project_prefs
            .iter()
            .find(|p| p.project_id == project_id)
            .cloned(),
    )
}

pub async fn admin_status(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Option<RoundStatusResponse>>) {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    let s = state.lock().unwrap();

    if !is_admin(&s, &session_id) {
        return (StatusCode::OK, Json(None));
    }

    (
        StatusCode::OK,
        Json(Some(RoundStatusResponse {
            round_number: s.round_number,
            round_open: s.round_open,
        })),
    )
}

pub async fn admin_round_start(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<bool>) {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    let mut s = state.lock().unwrap();

    if !is_admin(&s, &session_id) {
        return (StatusCode::UNAUTHORIZED, Json(false));
    }

    s.round_number += 1;
    s.round_open = true;

    let _ = s.save();
    (StatusCode::OK, Json(true))
}

pub async fn admin_round_close(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<bool>) {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    let mut s = state.lock().unwrap();

    if !is_admin(&s, &session_id) {
        return (StatusCode::UNAUTHORIZED, Json(false));
    }

    s.round_open = false;

    let _ = s.save();
    (StatusCode::OK, Json(true))
}

fn find_decision_mut<'a>(
    decisions: &'a mut [MatchDecision],
    round_number: u32,
    company_email: &str,
    group_email: &str,
    project_id: &str,
) -> Option<&'a mut MatchDecision> {
    decisions.iter_mut().find(|d| {
        d.round_number == round_number
            && d.company_email == company_email
            && d.group_email == group_email
            && d.project_id == project_id
    })
}

fn decision_status_for_user(role: &Role, d: &MatchDecision) -> String {
    if d.accepted_by_company && d.accepted_by_group {
        return "final".into();
    }

    match role {
        Role::Company => {
            if d.accepted_by_company {
                "accepted_by_me".into()
            } else if d.accepted_by_group {
                "accepted_by_other".into()
            } else {
                "pending".into()
            }
        }
        Role::Group => {
            if d.accepted_by_group {
                "accepted_by_me".into()
            } else if d.accepted_by_company {
                "accepted_by_other".into()
            } else {
                "pending".into()
            }
        }
        Role::Admin => "pending".into(),
    }
}

pub async fn me_match(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Vec<CurrentMatchResponse>>) {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    let s = state.lock().unwrap();

    let sess = match s.sessions.get(&session_id) {
        Some(x) => x.clone(),
        None => return (StatusCode::OK, Json(vec![])),
    };

    if !s.round_open {
        return (StatusCode::OK, Json(vec![]));
    }

    match sess.role {
        Role::Group => {
            if s.is_group_accepted(&sess.email) {
                return (StatusCode::OK, Json(vec![]));
            }
        }
        Role::Company => {}
        Role::Admin => {}
    }

    let matches = compute_matches_filtered(&s);

    let mut mine: Vec<MatchResult> = match sess.role {
        Role::Group => matches
            .into_iter()
            .filter(|m| m.group_email == sess.email)
            .take(1)
            .collect(),
        Role::Company => matches
            .into_iter()
            .filter(|m| m.company_email == sess.email)
            .collect(),
        Role::Admin => vec![],
    };

    mine.sort_by(|a, b| a.project_name.cmp(&b.project_name));

    let out = mine
        .into_iter()
        .filter_map(|m| {
            if s.is_project_full_or_accepted(&m.project_id) {
                return None;
            }
            if s.is_group_accepted(&m.group_email) {
                return None;
            }

            let decision = s.match_decisions.iter().find(|d| {
                d.round_number == s.round_number
                    && d.company_email == m.company_email
                    && d.group_email == m.group_email
                    && d.project_id == m.project_id
            });

            if decision.is_some_and(|d| d.rejected_by_company || d.rejected_by_group) {
                return None;
            }

            let status = decision
                .map(|d| decision_status_for_user(&sess.role, d))
                .unwrap_or_else(|| "pending".into());

            Some(CurrentMatchResponse {
                group_email: m.group_email,
                project_id: m.project_id,
                project_name: m.project_name,
                company_email: m.company_email,
                status: Some(status),
            })
        })
        .collect::<Vec<_>>();

    (StatusCode::OK, Json(out))
}

pub async fn me_final(
    State(state): State<Arc<Mutex<AppState>>>,
    Query(params): Query<HashMap<String, String>>,
) -> (StatusCode, Json<Vec<CurrentMatchResponse>>) {
    let session_id = params.get("session_id").cloned().unwrap_or_default();
    let s = state.lock().unwrap();

    let sess = match s.sessions.get(&session_id) {
        Some(x) => x.clone(),
        None => return (StatusCode::OK, Json(vec![])),
    };

    let mut finals: Vec<MatchResult> = match sess.role {
        Role::Group => s
            .accepted_matches
            .iter()
            .filter(|m| m.group_email == sess.email)
            .take(1)
            .cloned()
            .collect(),
        Role::Company => s
            .accepted_matches
            .iter()
            .filter(|m| m.company_email == sess.email)
            .cloned()
            .collect(),
        Role::Admin => vec![],
    };

    finals.sort_by(|a, b| a.project_name.cmp(&b.project_name));

    let out = finals
        .into_iter()
        .map(|m| CurrentMatchResponse {
            group_email: m.group_email,
            project_id: m.project_id,
            project_name: m.project_name,
            company_email: m.company_email,
            status: Some("final".into()),
        })
        .collect();

    (StatusCode::OK, Json(out))
}

pub async fn match_accept(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(body): Json<MatchActionRequest>,
) -> (StatusCode, Json<bool>) {
    let mut s = state.lock().unwrap();

    let sess = match s.sessions.get(&body.session_id) {
        Some(x) => x.clone(),
        None => return (StatusCode::UNAUTHORIZED, Json(false)),
    };

    if !s.round_open {
        return (StatusCode::BAD_REQUEST, Json(false));
    }

    let current = compute_matches_filtered(&s);

    let chosen = match sess.role {
        Role::Group => current.into_iter().find(|m| m.group_email == sess.email),
        Role::Company => {
            let pid = match &body.project_id {
                Some(x) if !x.trim().is_empty() => x.clone(),
                _ => return (StatusCode::BAD_REQUEST, Json(false)),
            };
            current
                .into_iter()
                .find(|m| m.company_email == sess.email && m.project_id == pid)
        }
        Role::Admin => None,
    };

    let m = match chosen {
        Some(x) => x,
        None => return (StatusCode::OK, Json(false)),
    };

    let round = s.round_number;

    if let Some(d) = find_decision_mut(
        &mut s.match_decisions,
        round,
        &m.company_email,
        &m.group_email,
        &m.project_id,
    ) {
        match sess.role {
            Role::Company => d.accepted_by_company = true,
            Role::Group => d.accepted_by_group = true,
            Role::Admin => {}
        }
    } else {
        let mut d = MatchDecision {
            round_number: round,
            company_email: m.company_email.clone(),
            group_email: m.group_email.clone(),
            project_id: m.project_id.clone(),
            accepted_by_company: false,
            accepted_by_group: false,
            rejected_by_company: false,
            rejected_by_group: false,
        };

        match sess.role {
            Role::Company => d.accepted_by_company = true,
            Role::Group => d.accepted_by_group = true,
            Role::Admin => {}
        }

        s.match_decisions.push(d);
    }

    if s.match_decisions.iter().any(|d| {
        d.round_number == round
            && d.company_email == m.company_email
            && d.group_email == m.group_email
            && d.project_id == m.project_id
            && d.accepted_by_company
            && d.accepted_by_group
    }) {
        let already = s
            .accepted_matches
            .iter()
            .any(|x| x.group_email == m.group_email && x.project_id == m.project_id);

        if !already {
            s.accepted_matches.push(m.clone());
        }
    }

    let _ = s.save();
    (StatusCode::OK, Json(true))
}

pub async fn match_reject(
    State(state): State<Arc<Mutex<AppState>>>,
    Json(body): Json<MatchActionRequest>,
) -> (StatusCode, Json<bool>) {
    let mut s = state.lock().unwrap();

    let sess = match s.sessions.get(&body.session_id) {
        Some(x) => x.clone(),
        None => return (StatusCode::UNAUTHORIZED, Json(false)),
    };

    if !s.round_open {
        return (StatusCode::BAD_REQUEST, Json(false));
    }

    let current = compute_matches_filtered(&s);

    let chosen = match sess.role {
        Role::Group => current.into_iter().find(|m| m.group_email == sess.email),
        Role::Company => {
            let pid = match &body.project_id {
                Some(x) if !x.trim().is_empty() => x.clone(),
                _ => return (StatusCode::BAD_REQUEST, Json(false)),
            };
            current
                .into_iter()
                .find(|m| m.company_email == sess.email && m.project_id == pid)
        }
        Role::Admin => None,
    };

    let m = match chosen {
        Some(x) => x,
        None => return (StatusCode::OK, Json(false)),
    };

    s.add_rejected(m.group_email.clone(), m.project_id.clone());

    let round = s.round_number;

    if let Some(d) = find_decision_mut(
        &mut s.match_decisions,
        round,
        &m.company_email,
        &m.group_email,
        &m.project_id,
    ) {
        match sess.role {
            Role::Company => d.rejected_by_company = true,
            Role::Group => d.rejected_by_group = true,
            Role::Admin => {}
        }
    } else {
        let mut d = MatchDecision {
            round_number: round,
            company_email: m.company_email.clone(),
            group_email: m.group_email.clone(),
            project_id: m.project_id.clone(),
            accepted_by_company: false,
            accepted_by_group: false,
            rejected_by_company: false,
            rejected_by_group: false,
        };

        match sess.role {
            Role::Company => d.rejected_by_company = true,
            Role::Group => d.rejected_by_group = true,
            Role::Admin => {}
        }

        s.match_decisions.push(d);
    }

    let _ = s.save();
    (StatusCode::OK, Json(true))
}

pub async fn round_status(State(state): State<Arc<Mutex<AppState>>>) -> Json<RoundStatusResponse> {
    let s = state.lock().unwrap();

    Json(RoundStatusResponse {
        round_number: s.round_number,
        round_open: s.round_open,
    })
}
