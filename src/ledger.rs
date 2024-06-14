#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LedgerItem {
    pub request: String,
    pub response: Response,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Response {
    Ok { code: u16, content: String },
    Err(String),
}
