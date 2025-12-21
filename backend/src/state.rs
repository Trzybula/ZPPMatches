use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use crate::models::{Group, Company};

#[derive(Serialize, Deserialize, Clone)]
pub struct AppState {
    pub groups: Vec<Group>,
    pub companies: Vec<Company>,
    pub sessions: HashMap<String, String>,
}

impl AppState {
    pub fn new() -> Self {
        AppState {
            groups: Vec::new(),
            companies: Vec::new(),
            sessions: HashMap::new(),
        }
    }
    
    pub fn save(&self) -> std::io::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write("state.json", json)
    }
}