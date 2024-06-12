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
    #[serde(with = "serde_yaml::with::singleton_map_recursive")]
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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Report {
    pub interval: Option<Duration>,
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
    pub report: Report,
    pub spec: Spec,
    pub behaviors: Behaviors,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Spec {
    Get {
        header: HashMap<String, Vec<ValueParser>>,
        query: HashMap<String, Vec<QueryValueParser>>,
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
pub enum QueryValueParser {
    Static(String),
    Env(String),
    Increment { start: usize, step: usize },
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Behaviors {
    pub ok: Vec<Behavior>,
    pub error: ErrorBehavior,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ErrorBehavior {
    pub backoff: Option<Duration>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Behavior {
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
