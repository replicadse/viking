use std::collections::HashMap;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WithVersion {
    pub version: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub version: String,
    pub campaigns: HashMap<String, Campaign>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Campaign {
    pub phases: Vec<Phase>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Duration {
    #[serde(rename = "ms")]
    MilliSeconds(u64),
    #[serde(rename = "s")]
    Seconds(u64),
}

impl Duration {
    pub fn to_ms(&self) -> u64 {
        match self {
            | Duration::MilliSeconds(v) => *v,
            | Duration::Seconds(v) => v * 1000,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Phase {
    pub target: ValueParser,
    pub threads: usize,
    pub ends: End,
    pub timeout: Duration,
    pub spec: Spec,
    pub behaviours: Behaviours,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Spec {
    Get {
        header: HashMap<String, Vec<ValueParser>>,
        query: HashMap<String, Vec<ValueParser>>,
    },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueParser {
    Static(String),
    Env(String),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Behaviours {
    pub ok: Vec<Behaviour>,
    pub error: ErrorBehaviour,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorBehaviour {
    Backoff(u64),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Behaviour {
    #[serde(rename = "match")]
    pub match_: String,
    pub mark: Mark,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mark {
    Success,
    Error,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct End {
    pub requests: Option<usize>,
    pub time: Option<Duration>,
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_deserialize() {
        serde_yaml::from_str::<Config>(include_str!("../res/example.yaml")).unwrap();
    }
}
