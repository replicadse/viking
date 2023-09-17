use {
    crate::config::{Campaign, ErrorBehaviour, Mark, Spec, ValueParser},
    anyhow::Result,
    fancy_regex::Regex,
    itertools::Itertools,
    reqwest::{
        header::{HeaderMap, HeaderName, HeaderValue},
        Method, StatusCode,
    },
    std::{
        collections::BTreeMap,
        thread::{spawn, JoinHandle},
        time::Duration,
    },
};

pub struct Engine {}

impl Engine {
    pub async fn raid(&self, campaign: &Campaign) -> Result<()> {
        #[derive(Debug)]
        enum ThreadEvent {
            Success { status_code: StatusCode },
            Error {},
        }
        #[derive(Debug)]
        struct ThreadStats {
            count: usize,
            success: usize,
            error: usize,
            client_error: usize,
        }

        let raid_start = std::time::Instant::now();

        for phase in &campaign.phases {
            let phase_start = std::time::Instant::now();
            let (tasks_tx, tasks_rx) =
                flume::bounded::<(Method, String, HeaderMap, Vec<(String, String)>, Duration)>(phase.threads * 2);
            let (status_tx, status_rx) = flume::bounded::<(usize, ThreadEvent)>(phase.threads * 2);

            let mut threads = Vec::<JoinHandle<_>>::with_capacity(phase.threads);
            let mut thread_stats = BTreeMap::<usize, ThreadStats>::new();
            for t_idx in 0..phase.threads {
                let thread_rx = tasks_rx.clone();
                let thread_status_tx = status_tx.clone();
                let on_error = phase.behaviours.error.clone();

                let thread = spawn(move || {
                    let client = reqwest::blocking::Client::new();
                    for msg in thread_rx.iter() {
                        let req = client
                            .request(msg.0, msg.1)
                            .headers(msg.2)
                            .query::<Vec<(String, String)>>(&msg.3)
                            .timeout(msg.4);
                        let response = req.send();

                        match response {
                            | Ok(v) => {
                                thread_status_tx
                                    .send((
                                        t_idx,
                                        ThreadEvent::Success {
                                            status_code: v.status(),
                                        },
                                    ))
                                    .unwrap();
                            },
                            | Err(_) => {
                                thread_status_tx.send((t_idx, ThreadEvent::Error {})).unwrap();
                                match on_error {
                                    | ErrorBehaviour::Backoff(v) => std::thread::sleep(Duration::from_millis(v)),
                                };
                            },
                        }
                    }
                });
                // consumer threads
                threads.push(thread);
                thread_stats.insert(
                    t_idx,
                    ThreadStats {
                        count: 0,
                        success: 0,
                        error: 0,
                        client_error: 0,
                    },
                );
            }
            drop(tasks_rx);
            drop(status_tx);

            match &phase.spec {
                | Spec::Get { header, query } => {
                    let header_map = HeaderMap::from_iter(
                        header
                            .iter()
                            .map(|v| {
                                (
                                    v.0.parse().unwrap(),
                                    v.1.into_iter()
                                        .map(|v| match v {
                                            | ValueParser::Static(v) => v.to_owned(),
                                            | ValueParser::Env(v) => std::env::var(v).unwrap(),
                                        })
                                        .join(",")
                                        .parse()
                                        .unwrap(),
                                )
                            })
                            .collect::<Vec<(HeaderName, HeaderValue)>>(),
                    );
                    let query_map = query
                        .iter()
                        .map(|v| {
                            (
                                v.0.clone(),
                                v.1.into_iter()
                                    .map(|v| match v {
                                        | ValueParser::Static(v) => v.to_owned(),
                                        | ValueParser::Env(v) => std::env::var(v).unwrap(),
                                    })
                                    .join(","),
                            )
                        })
                        .collect::<Vec<_>>();

                    let target = match &phase.target {
                        | ValueParser::Static(v) => v.to_owned(),
                        | ValueParser::Env(v) => std::env::var(v).unwrap(),
                    };

                    let timeout_ms = phase.timeout.to_ms();
                    let cond_req = phase.ends.requests.clone();
                    let cond_time = phase.ends.time.clone();

                    spawn(move || {
                        let mut req_idx = 0_usize;
                        let start = std::time::Instant::now();
                        loop {
                            if let Some(v) = &cond_req {
                                if req_idx >= *v {
                                    break;
                                }
                            }
                            if let Some(v) = &cond_time {
                                if start.elapsed().as_millis() >= v.to_ms() as u128 {
                                    break;
                                }
                            }

                            tasks_tx
                                .send((
                                    Method::GET,
                                    target.clone(),
                                    header_map.clone(),
                                    query_map.clone(),
                                    Duration::from_millis(timeout_ms),
                                ))
                                .unwrap();
                            req_idx += 1;
                        }
                    });
                },
            };

            let mut behaviours = Vec::<(Regex, &Mark)>::new();
            for behav in &phase.behaviours.ok {
                behaviours.push((Regex::new(&behav.match_).unwrap(), &behav.mark));
            }

            for msg in status_rx.iter() {
                let stats = &mut thread_stats.get_mut(&msg.0).unwrap();
                match msg.1 {
                    | ThreadEvent::Success { status_code } => {
                        stats.count += 1;
                        let s_code = status_code.as_u16().to_string();
                        for b in &behaviours {
                            if b.0.is_match(&s_code).unwrap() {
                                match b.1 {
                                    | Mark::Success => stats.success += 1,
                                    | Mark::Error => stats.error += 1,
                                }
                                break;
                            }
                        }
                    },
                    | ThreadEvent::Error {} => {
                        stats.count += 1;
                        stats.client_error += 1;
                    },
                };

                eprintln!(
                    "Thread #{}: Count: {}, OK: {}, Error: {}, Client Error: {}",
                    msg.0, stats.count, stats.success, stats.error, stats.client_error
                );
            }

            for t in threads {
                t.join().unwrap();
            }

            let phase_elapsed = phase_start.elapsed();
            eprintln!("");
            eprintln!("=== === ===");
            eprintln!(
                "Phase with {} requests",
                thread_stats.iter().map(|v| v.1.count).sum::<usize>()
            );
            eprintln!("\ttook {}s ({}ms)", phase_elapsed.as_secs(), phase_elapsed.as_millis());
            eprintln!(
                "\tavg {:.2} requests / second",
                thread_stats.iter().map(|v| v.1.count).sum::<usize>() as f32 / phase_elapsed.as_secs_f32()
            );
            eprintln!(
                "\tavg {:.2} requests / second / thread",
                thread_stats.iter().map(|v| v.1.count).sum::<usize>() as f32
                    / phase_elapsed.as_secs_f32()
                    / phase.threads as f32
            );
            eprintln!(
                "\tOK: {}, Error: {}, Client error: {}",
                thread_stats.iter().map(|v| v.1.success).sum::<usize>(),
                thread_stats.iter().map(|v| v.1.error).sum::<usize>(),
                thread_stats.iter().map(|v| v.1.client_error).sum::<usize>(),
            );
        }

        let raid_elapsed = raid_start.elapsed();
        eprintln!("");
        eprintln!("=== === ===");
        eprintln!(
            "Raid took {} seconds ({} ms).",
            raid_elapsed.as_secs(),
            raid_elapsed.as_millis()
        );

        Ok(())
    }
}
