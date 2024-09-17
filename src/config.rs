use std::fs;
use serde::{Deserialize, Serialize};

fn get_default_cps() -> Option<usize> { Some(1) }
fn get_default_url() -> String { "https://10.199.28.159".to_string() }
fn get_default_max_connections() -> Option<u32> { None }
fn get_default_max_time() -> Option<u32> { None }
fn get_default_thread_count() -> usize { 1 }
fn get_default_request_timeout_sec() -> u64 { 10 }
fn get_default_check_cert() -> bool { false }
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

    #[serde(default = "get_default_thread_count")]
    pub thread_count: usize,

    #[serde(default = "get_default_request_timeout_sec")]
    pub request_timeout_sec: u64,

    #[serde(default = "get_default_check_cert")]
    pub check_cert: bool,

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