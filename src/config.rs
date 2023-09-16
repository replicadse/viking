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
pub struct Phase {
    pub target: String,
    pub threads: usize,
    pub ends: End,
    pub timeout_ms: u64,
    pub report: Report,
    pub spec: Spec,
    // pub behaviours: Vec<Behaviour>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Report {
    pub interval: Interval,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interval {
    Seconds(usize),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Spec {
    Get {
        header: HashMap<String, Vec<String>>,
        query: HashMap<String, Vec<String>>,
    },
}

// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub struct Behaviour {
//     #[serde(rename = "match")]
//     pub match_: String,
//     pub mark: Mark,
// }

// #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
// #[serde(rename_all = "snake_case")]
// pub enum Mark {
//     Success,
//     Error,
// }

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum End {
    Requests(usize),
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_deserialize() {
        serde_yaml::from_str::<Config>(include_str!("../res/example.yaml")).unwrap();
    }
}
