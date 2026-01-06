use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Group,
    Company,
    Admin,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AuthUser {
    pub email: String,
    pub password: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Group {
    pub name: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Company {
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
pub struct GroupPreferences {
    pub group_email: String,
    pub project_ids_ranked: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectPreferences {
    pub project_id: String,
    pub group_emails_ranked: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchResult {
    pub group_email: String,
    pub project_id: String,
    pub project_name: String,
    pub company_email: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct MatchDecision {
    pub round_number: u32,
    pub company_email: String,
    pub group_email: String,
    pub project_id: String,
    pub accepted_by_company: bool,
    pub accepted_by_group: bool,
    pub rejected_by_company: bool,
    pub rejected_by_group: bool,
}
