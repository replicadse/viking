use {
    crate::{
        config::{
            Campaign,
            Mark,
            Spec,
            ValueParser,
            VarsValueParser,
        },
        ledger::{
            LedgerItem,
            Response,
        },
    },
    anyhow::Result,
    crossterm::terminal::{
        Clear,
        ClearType,
    },
    fancy_regex::Regex,
    reqwest::{
        header::{
            HeaderMap,
            HeaderName,
            HeaderValue,
        },
        Method,
        StatusCode,
    },
    std::{
        collections::{
            BTreeMap,
            HashMap,
        },
        thread::{
            spawn,
            JoinHandle,
        },
        time::Duration,
    },
};

#[derive(Debug)]
struct ThreadStats {
    count: usize,
    success: usize,
    error: usize,
    client_error: usize,
}

pub struct Engine {}

impl Engine {
    pub async fn raid(&self, campaign: &Campaign, recorder: Option<flume::Sender<String>>) -> Result<()> {
        #[derive(Debug)]
        enum ThreadEvent {
            Success { status_code: StatusCode },
            Error {},
        }

        let raid_start = std::time::Instant::now();

        for phase in &campaign.phases {
            let phase_start = std::time::Instant::now();
            let (tasks_tx, tasks_rx) = flume::bounded::<(Method, String, HeaderMap, Duration)>(phase.threads * 2);
            let (status_tx, status_rx) = flume::bounded::<(usize, ThreadEvent)>(phase.threads * 2);

            let mut threads = Vec::<JoinHandle<_>>::with_capacity(phase.threads);
            let mut thread_stats = BTreeMap::<usize, ThreadStats>::new();
            for t_idx in 0..phase.threads {
                let thread_rx = tasks_rx.clone();
                let thread_status_tx = status_tx.clone();
                let on_error = phase.behaviors.error.clone();
                let thread_recorder = recorder.clone();
                let thread = spawn(move || {
                    let client = reqwest::blocking::Client::new();
                    for msg in thread_rx.iter() {
                        let req = client.request(msg.0, &msg.1).headers(msg.2).timeout(msg.3);
                        let response = req.send();

                        match response {
                            | Ok(v) => {
                                thread_status_tx
                                    .send((t_idx, ThreadEvent::Success {
                                        status_code: v.status(),
                                    }))
                                    .unwrap();

                                if let Some(recorder) = &thread_recorder {
                                    recorder
                                        .send(
                                            serde_json::to_string(&LedgerItem {
                                                request: msg.1.clone(),
                                                response: Response::Ok {
                                                    code: v.status().as_u16(),
                                                    content: v.text().unwrap(),
                                                },
                                            })
                                            .unwrap(),
                                        )
                                        .unwrap();
                                }
                            },
                            | Err(e) => {
                                thread_status_tx.send((t_idx, ThreadEvent::Error {})).unwrap();

                                if let Some(recorder) = &thread_recorder {
                                    recorder
                                        .send(
                                            serde_json::to_string(&LedgerItem {
                                                request: msg.1.clone(),
                                                response: Response::Err(format!("{:?}", e)),
                                            })
                                            .unwrap(),
                                        )
                                        .unwrap();
                                }

                                if let Some(v) = &on_error.backoff {
                                    std::thread::sleep(Duration::from_millis(v.to_ms()));
                                }
                            },
                        }
                    }
                });
                // consumer threads
                threads.push(thread);
                thread_stats.insert(t_idx, ThreadStats {
                    count: 0,
                    success: 0,
                    error: 0,
                    client_error: 0,
                });
            }
            drop(tasks_rx);
            drop(status_tx);

            match &phase.spec {
                | Spec::Get { headers: header, vars } => {
                    let header_map = HeaderMap::from_iter(
                        header
                            .iter()
                            .map(|v| {
                                (
                                    v.0.parse().unwrap(),
                                    match v.1 {
                                        | ValueParser::Static(v) => v.to_owned(),
                                        | ValueParser::Env(v) => std::env::var(v).unwrap(),
                                    }
                                    .parse()
                                    .unwrap(),
                                )
                            })
                            .collect::<Vec<(HeaderName, HeaderValue)>>(),
                    );

                    let mut hb = handlebars::Handlebars::new();
                    hb.set_strict_mode(true);

                    let mut vars_map = vars
                        .iter()
                        .map(|v| (v.0.clone(), VarsValueParserState::from(v.1.clone())))
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

                            let mut vars_map_rendered = HashMap::<&str, String>::new();
                            for v in &mut vars_map {
                                vars_map_rendered.insert(&v.0, v.1.access_string());
                            }

                            let target = hb.render_template(&target, &vars_map_rendered).unwrap();
                            let payload = (
                                Method::GET,
                                target,
                                header_map.clone(),
                                Duration::from_millis(timeout_ms),
                            );
                            tasks_tx.send(payload).unwrap();
                            req_idx += 1;
                        }
                    });
                },
            };

            let mut behaviors = Vec::<(Regex, &Mark)>::new();
            for behav in &phase.behaviors.ok {
                behaviors.push((Regex::new(&behav.match_).unwrap(), &behav.mark));
            }

            let mut report_timer = std::time::Instant::now();
            self.report(&thread_stats, phase_start.elapsed());
            for msg in status_rx.iter() {
                let stats = &mut thread_stats.get_mut(&msg.0).unwrap();
                match msg.1 {
                    | ThreadEvent::Success { status_code } => {
                        stats.count += 1;
                        let s_code = status_code.as_u16().to_string();
                        for b in &behaviors {
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

                if let Some(v) = &phase.report.interval {
                    if report_timer.elapsed().as_millis() > v.to_ms() as u128 {
                        self.report(&thread_stats, phase_start.elapsed());
                        report_timer = std::time::Instant::now();
                    }
                } else {
                    self.report(&thread_stats, phase_start.elapsed());
                    report_timer = std::time::Instant::now();
                }
            }

            self.report(&thread_stats, phase_start.elapsed());

            for t in threads {
                t.join().unwrap();
            }
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

    fn report(&self, data: &BTreeMap<usize, ThreadStats>, elapsed: Duration) {
        let stdout = &mut std::io::stdout();
        crossterm::execute!(stdout, Clear(ClearType::All)).unwrap();
        eprintln!("Stats:");
        eprintln!("{} requests", data.iter().map(|v| v.1.count).sum::<usize>());
        eprintln!("{:.2}s elapsed", elapsed.as_secs_f32());
        eprintln!(
            "avg {:.2} requests / second",
            data.iter().map(|v| v.1.count).sum::<usize>() as f32 / elapsed.as_secs_f32()
        );
        eprintln!(
            "avg {:.2} requests / second / thread",
            data.iter().map(|v| v.1.count).sum::<usize>() as f32 / elapsed.as_secs_f32() / data.len() as f32
        );
        eprintln!(
            "OK: {}, Error: {}, Client error: {}",
            data.iter().map(|v| v.1.success).sum::<usize>(),
            data.iter().map(|v| v.1.error).sum::<usize>(),
            data.iter().map(|v| v.1.client_error).sum::<usize>(),
        );
        eprintln!("");
        eprintln!("Thread details:");
        for d in data {
            eprintln!(
                "Thread #{}:\tTotal: {}\tOK: {}\tError: {}\tRequest Error: {}",
                d.0, d.1.count, d.1.success, d.1.error, d.1.client_error
            )
        }
    }
}

enum VarsValueParserState {
    String(String),
    Increment { state: usize, step: usize },
}
impl VarsValueParserState {
    pub fn access_string(&mut self) -> String {
        match self {
            | Self::String(v) => v.clone(),
            | Self::Increment { state, step } => {
                let v = *state;
                *self = Self::Increment {
                    state: (*state) + (*step),
                    step: *step,
                };
                v.to_string()
            },
        }
    }
}
impl From<VarsValueParser> for VarsValueParserState {
    fn from(value: VarsValueParser) -> Self {
        match value {
            | VarsValueParser::Static(v) => Self::String(v),
            | VarsValueParser::Env(v) => Self::String(std::env::var(v).unwrap()),
            | VarsValueParser::Increment { start, step } => Self::Increment { state: start, step },
        }
    }
}
