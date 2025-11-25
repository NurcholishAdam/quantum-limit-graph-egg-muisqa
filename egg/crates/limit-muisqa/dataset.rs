use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct MuISQAEntry {
    pub id: String,
    pub domain: String,          // biology, chemistry, physics, etc.
    pub question: String,
    pub sub_intents: Vec<String>,
    pub answers: Vec<String>,
}

pub fn load_dataset(path: &str) -> anyhow::Result<Vec<MuISQAEntry>> {
    let data = fs::read_to_string(path)?;
    let entries: Vec<MuISQAEntry> = serde_json::from_str(&data)?;
    Ok(entries)
}
