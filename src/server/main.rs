pub mod account;
pub mod bank;
pub mod runner;
pub mod server;
pub mod services;
pub mod traits;
pub mod transaction;
pub mod user;


use log::*;
use std::sync::{Arc, Mutex};

const IP: &str = "127.0.0.1:8080";
const RUNNER_SLEEP_TIME : u64 = 24*60*60; // 24 hours

fn main() {
    env_logger::init();
    let bank_server = Arc::new(Mutex::new(server::Server::new()));

    let mut runner = runner::ServerRunner::new(&bank_server);
    runner.run(std::time::Duration::from_secs(RUNNER_SLEEP_TIME));

    info!("Starting HTTP server...");
    rouille::start_server(IP, move |req| {
        bank_server.lock().unwrap().handle_request(req)
    });

}
