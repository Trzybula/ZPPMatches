use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Company {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Group {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Project {
    pub id: String,
    pub company_email: String,
    pub name: String,
    pub description: Option<String>,
    pub capacity: u32,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchResult {
    pub group_email: String,
    pub project_id: String,
    pub project_name: String,
    pub company_email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RoundStatusResponse {
    pub round_number: u32,
    pub round_open: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CurrentMatchResponse {
    pub group_email: String,
    pub project_id: String,
    pub project_name: String,
    pub company_email: String,
    pub status: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoginResponse {
    pub ok: bool,
    pub message: String,
    pub session_id: Option<String>,
    pub email: Option<String>,
    pub role: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RegisterResponse {
    pub ok: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateGroupRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateCompanyRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct CreateProjectRequest {
    pub id: String,
    pub company_email: String,
    pub name: String,
    pub description: Option<String>,
    pub capacity: u32,
    pub active: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GroupPreferencesResponse {
    pub group_email: String,
    pub project_ids_ranked: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectPreferencesResponse {
    pub project_id: String,
    pub group_emails_ranked: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SetGroupPreferencesRequest {
    pub session_id: String,
    pub project_ids_ranked: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SetProjectPreferencesRequest {
    pub session_id: String,
    pub project_id: String,
    pub group_emails_ranked: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchActionRequest {
    pub session_id: String,
    pub project_id: Option<String>,
}
