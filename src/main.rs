mod config;
mod loader;
mod ui;

use crate::config::config_read;
use chrono::Local;
use circular_buffer::CircularBuffer;
use env_logger::Builder;
use log::LevelFilter;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use std::ops::Deref;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering::Relaxed;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task;
use tokio::time; // 1.3.0

#[derive(Serialize, Deserialize, Debug)]
struct Statistic {
    resp_code: HashMap<u16, usize>,
    other_err: usize,
    cps: usize,

    #[serde(skip_serializing)]
    #[serde(skip_deserializing)]
    error_log: CircularBuffer<256, String>,
}

impl Statistic {
    fn new() -> Self {
        Statistic {
            resp_code: HashMap::new(),
            other_err: 0,
            cps: 0,
            error_log: CircularBuffer::new(),
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    log_init();
    let config = config_read();

    let mut handlers = vec![];
    let semaphore = Arc::new(Semaphore::new(0));
    let inprogress_statistic: Arc<RwLock<Statistic>> = Arc::new(RwLock::new(Statistic::new()));
    let ui_statistic: Arc<RwLock<Statistic>> = Arc::new(RwLock::new(Statistic::new()));
    let stop = Arc::new(AtomicBool::new(false));
    let url = config.url;

    // test request
    {
        let client = Client::builder()
            .timeout(Duration::from_secs(config.request_timeout_sec))
            .danger_accept_invalid_hostnames(config.check_cert)
            .danger_accept_invalid_certs(config.check_cert);

        let client = client.build().unwrap();

        let resp = client.get(&url).send().await;
        let mut w = inprogress_statistic.write().unwrap();

        match resp {
            Ok(resp) => {
                let resp_code = resp.status().as_u16();
                *w.resp_code.entry(resp_code).or_insert(0) += 1;
                w.cps += 1;
                //w.error_log.push_back(format!("{}| {} - {}", Local::now().format("%H:%M:%S|"), "ok", url));
            }
            Err(err) => {
                return Err(err.into());
                //if err.is_request() {
                //    stop.store(true, Relaxed);
                //    w.error_log.push_back(format!("{}| {}", Local::now().format("%H:%M:%S"), "Остановка запросов из-за ошибки"));
                //}
            }
        }
    }

    for _ in 0..config.thread_count {
        let semaphore = Arc::clone(&semaphore);
        let inprogress_statistic = Arc::clone(&inprogress_statistic);
        let stop = Arc::clone(&stop);
        let url = url.clone();

        let h = task::spawn(async move {
            //log::info!("Создание потока нагрузки");

            let client = Client::builder()
                .timeout(Duration::from_secs(config.request_timeout_sec))
                .danger_accept_invalid_hostnames(config.check_cert)
                .danger_accept_invalid_certs(config.check_cert);

            let client = client.build().unwrap();

            loop {
                if config.cps.is_some() {
                    let p = semaphore.acquire().await.unwrap();
                    p.forget();
                }

                if stop.load(Relaxed) == true {
                    break;
                }

                let resp = client.get(&url).send().await;
                let mut w = inprogress_statistic.write().unwrap();

                match resp {
                    Ok(resp) => {
                        let resp_code = resp.status().as_u16();
                        *w.resp_code.entry(resp_code).or_insert(0) += 1;
                        w.cps += 1;
                        //w.error_log.push_back(format!("{}| {} - {}", Local::now().format("%H:%M:%S|"), "ok", url));
                    }
                    Err(err) => {
                        w.error_log.push_back(format!(
                            "{}| {}",
                            Local::now().format("%H:%M:%S"),
                            err
                        ));
                        w.other_err += 1;

                        //if err.is_request() {
                        //    stop.store(true, Relaxed);
                        //    w.error_log.push_back(format!("{}| {}", Local::now().format("%H:%M:%S"), "Остановка запросов из-за ошибки"));
                        //}
                    }
                }
            }
            //log::info!("Завершение потока нагрузки");
        });

        handlers.push(h);
    }

    {
        let stop = Arc::clone(&stop);
        let h = task::spawn(async move {
            //log::info!("Создание потока выдающего квоты на запросы");
            let mut interval = time::interval(Duration::from_millis(1000));

            loop {
                interval.tick().await;

                if stop.load(Relaxed) == true {
                    semaphore.forget_permits(Semaphore::MAX_PERMITS);
                    semaphore.add_permits(Semaphore::MAX_PERMITS);
                    break;
                }

                if let Some(cps) = config.cps {
                    semaphore.forget_permits(Semaphore::MAX_PERMITS);
                    semaphore.add_permits(cps);
                }
            }
            //log::info!("Завершение потока выдающего квоты на запросы");
        });
        handlers.push(h);
    }

    {
        let stop = Arc::clone(&stop);
        let inprogress_statistic = Arc::clone(&inprogress_statistic);
        let ui_statistic = Arc::clone(&ui_statistic);
        let h = task::spawn(async move {
            //log::info!("Создание потока статистики");
            let mut interval = time::interval(Duration::from_millis(1000));
            let mut stop_counter = 5;

            loop {
                interval.tick().await;

                if stop.load(Relaxed) == true {
                    stop_counter -= 1;

                    if stop_counter == 0 {
                        break;
                    }
                }

                let mut iw = inprogress_statistic.write().unwrap();
                let mut uw = ui_statistic.write().unwrap();
                if !config.ui {
                    //log::info!("{:?}", iw.deref());
                }

                uw.cps = iw.cps;
                uw.resp_code = iw.resp_code.clone();
                uw.other_err = iw.other_err;
                uw.error_log = iw.error_log.clone();

                iw.cps = 0;
            }
            // log::info!("Завершение потока статистики");
        });
        handlers.push(h);
    }

    {
        let stop = Arc::clone(&stop);
        let ui_statistic = Arc::clone(&ui_statistic);
        let h = task::spawn(async move {
            //log::info!("Создание потока UI");
            ui::ui_main(config.ui, ui_statistic).unwrap();
            stop.store(true, Relaxed);
            //log::info!("Завершение потока UI");
        });

        handlers.push(h);
    }

    for h in handlers {
        h.await.unwrap();
    }

    Ok(())
}

fn log_init() {
    Builder::new()
        .format(|buf, record| {
            writeln!(
                buf,
                "{} [{}] {:?} - {}",
                Local::now().format("%Y-%m-%dT%H:%M:%S"),
                record.level(),
                thread::current().id(),
                record.args()
            )
        })
        .filter(None, LevelFilter::Info)
        .init();
}
