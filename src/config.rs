use std::collections::HashMap;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct WithVersion {
    pub version: String,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub version: String,
    pub campaigns: HashMap<String, Campaign>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Campaign {
    pub phases: Vec<Phase>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Phase {
    pub target: String,
    pub threads: usize,
    pub report: Report,
    pub spec: Spec,
    pub behaviours: Vec<Behaviour>,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Report {
    pub interval: Interval,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interval {
    Seconds(usize),
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Spec {
    Get {
        header: HashMap<String, Vec<String>>,
        query: HashMap<String, Vec<String>>,
    },
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Behaviour {
    #[serde(rename = "match")]
    pub match_: String,
    pub mark: Mark,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Mark {
    Success,
    Error,
}

mod test {
    use super::*;

    #[tokio::test]
    async fn test_deserialize() {
        serde_yaml::from_str::<Config>(include_str!("../res/example.yaml")).unwrap();
    }
}
