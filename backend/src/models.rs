use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Group {
    pub name: String,
    pub email: String,
    pub password: String,
    pub preferences: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Company {
    pub name: String,
    pub email: String,
    pub password: String,
    pub preferences: Vec<String>,
}

#[derive(Deserialize)]
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
pub struct AddPref {
    pub session_id: String,
    pub pref: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchResult {
    pub group: String,
    pub company: String,
}