use std::fs;
use serde::{Deserialize, Serialize};

fn get_default_cps() -> Option<usize> { None }
fn get_default_url() -> String { "https://google.com".to_string() }
fn get_default_max_connections() -> Option<u32> { None }
fn get_default_max_time() -> Option<u32> { None }
fn get_default_max_threads() -> usize { num_cpus::get() }
fn get_default_request_timeout_sec() -> u64 { 10 }
fn get_default_ui() -> bool { true }


#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    #[serde(default = "get_default_cps")]
    pub cps: Option<usize>,

    #[serde(default = "get_default_url")]
    pub url: String,

    #[serde(default = "get_default_max_connections")]
    pub max_connections: Option<u32>,

    #[serde(default = "get_default_max_time")]
    pub max_time: Option<u32>,

    #[serde(default = "get_default_max_threads")]
    pub max_threads: usize,

    #[serde(default = "get_default_request_timeout_sec")]
    pub request_timeout_sec: u64,

    #[serde(default = "get_default_ui")]
    pub ui: bool,
}


pub fn config_read() -> Config {
    let mut config: Config = serde_json::from_str("{}").unwrap();

    if let Ok(data) = fs::read_to_string("config.json") {
        config = serde_json::from_str(data.as_str()).unwrap();
    }

    println!("{:?}", config);
    config
}