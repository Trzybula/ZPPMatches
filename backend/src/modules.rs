use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Group {
    pub id: u32,
    pub name: String,
    pub preferences: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Company {
    pub id: u32,
    pub name: String,
    pub preferences: Vec<u32>,
}

#[derive(Serialize, Deserialize)]
pub struct MatchResult {
    pub pairs: Vec<(u32, u32)>,
}

#[derive(Deserialize)]
pub struct UpdatePrefs {
    pub email: String,
    pub preferences: Vec<String>,
}
