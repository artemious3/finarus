pub mod account;
pub mod bank;
pub mod runner;
pub mod server;
pub mod services;
pub mod traits;
pub mod transaction;
pub mod user;


use log::*;

const IP: &str = "127.0.0.1:8080";

fn main() {
    env_logger::init();
    let bank_server = server::Server::new();


    info!("Starting HTTP server...");
    rouille::start_server(IP, move |req| {
        bank_server.lock().unwrap().handle_request(req)
    });

}
