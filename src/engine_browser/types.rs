use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowserTabInfo {
    pub url: String,
    pub title: String,
    pub domain: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BrowserCommand {
    pub action: String,
    pub text: Option<String>,
    pub url: Option<String>,
}
