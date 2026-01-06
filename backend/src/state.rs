use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::models::{
    AuthUser, Company, Group, GroupPreferences, MatchDecision, MatchResult, Project,
    ProjectPreferences, Role,
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Session {
    pub email: String,
    pub role: Role,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AppState {
    pub users: Vec<AuthUser>,
    pub groups: Vec<Group>,
    pub companies: Vec<Company>,
    pub projects: Vec<Project>,
    pub group_prefs: Vec<GroupPreferences>,
    pub project_prefs: Vec<ProjectPreferences>,
    #[serde(default)]
    pub sessions: HashMap<String, Session>,
    pub round_number: u32,
    pub round_open: bool,
    #[serde(default)]
    pub current_matches: Vec<MatchResult>,
    #[serde(default)]
    pub accepted_matches: Vec<MatchResult>,
    #[serde(default)]
    pub rejected_pairs: Vec<(String, String)>,
    #[serde(default)]
    pub match_decisions: Vec<MatchDecision>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            users: Vec::new(),
            groups: Vec::new(),
            companies: Vec::new(),
            projects: Vec::new(),
            group_prefs: Vec::new(),
            project_prefs: Vec::new(),
            sessions: HashMap::new(),
            round_number: 0,
            round_open: false,
            current_matches: Vec::new(),
            accepted_matches: Vec::new(),
            rejected_pairs: Vec::new(),
            match_decisions: Vec::new(),
        }
    }
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write("state.json", json)?;
        Ok(())
    }

    pub fn is_rejected(&self, group_email: &str, project_id: &str) -> bool {
        self.rejected_pairs
            .iter()
            .any(|(g, p)| g == group_email && p == project_id)
    }

    pub fn add_rejected(&mut self, group_email: String, project_id: String) {
        if !self.is_rejected(&group_email, &project_id) {
            self.rejected_pairs.push((group_email, project_id));
        }
    }

    pub fn is_group_accepted(&self, group_email: &str) -> bool {
        self.accepted_matches
            .iter()
            .any(|m| m.group_email == group_email)
    }

    pub fn is_project_full_or_accepted(&self, project_id: &str) -> bool {
        self.accepted_matches
            .iter()
            .any(|m| m.project_id == project_id)
    }
}
