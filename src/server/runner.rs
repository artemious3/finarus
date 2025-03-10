/*
 * Server runner updates dynamic properties of bank system,
 * such as credits, loans, deposits.
 */

use crate::server::Server;
use crate::traits::dynamic::Dynamic;
use chrono::{DateTime, Utc};
use log::*;
use std::option::Option;
use std::sync::{Arc, Mutex, Weak};

pub struct ServerRunner {
    time: Arc<Mutex<DateTime<Utc>>>,
    thread_handle: Option<std::thread::Thread>,
}

impl ServerRunner {
    pub fn new() -> Self {
        ServerRunner {
            time: Arc::new(Mutex::new(Utc::now())),
            thread_handle: None,
        }
    }

    pub fn run(&mut self, serv: &Arc<Mutex<Server>>, timeout: std::time::Duration) {
        let weak_server = Arc::downgrade(serv);
        let local_time = self.time.clone();
        self.thread_handle = Some(
            std::thread::spawn(move || loop {
                info!("Server runner woke up. Good morning!");
                let maybe_server = weak_server.upgrade();
                match maybe_server {
                    None => {
                        info!("Server is dead, and so am I. Good bye!");
                        return;
                    }
                    Some(mtx_server) => {
                        info!("Server is alive. Performing dynamic update...");
                        let mut server = mtx_server.lock().expect("Mutex");
                        server.update(&local_time.lock().expect("Mutex"));
                    }
                }
                info!(
                    "Dynamic update performed successfully. Going asleep for {}s. Good night.",
                    timeout.as_secs()
                );
                std::thread::park_timeout(timeout);
            })
            .thread()
            .clone(),
        )
    }

    pub fn force_wakeup(&self) {
        let handle = self.thread_handle.clone().expect("Not running");
        handle.unpark();
    }

    pub fn advance_time(&mut self, time: DateTime<Utc>) -> Result<(), &str> {
        let mut time_ref = self.time.lock().expect("Mutex");
        if time < *time_ref {
            Err("Can't advance time in past")
        } else {
            *time_ref = time;
            Ok(())
        }
    }
}
