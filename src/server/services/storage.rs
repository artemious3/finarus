use crate::server::Server;
use crate::traits::storable::Storable;
use log::*;
use std::sync::{Arc, Mutex, Weak};

const STORAGE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(60 * 5);

pub struct StorageService {
    server: Weak<Mutex<Server>>,
}

impl StorageService {
    pub fn new(serv: &Arc<Mutex<Server>>) -> Self {
        StorageService {
            server: Arc::downgrade(serv),
        }
    }

    pub fn run(&mut self) {
        let local_server = self.server.clone();
        std::thread::spawn(move || {
            log::info!("This is you cuting-edge noSQL BD - StorageService!");
            log::info!("StorageService thread spawned");
            loop {
                std::thread::park_timeout(STORAGE_TIMEOUT);
                log::info!("Good morning!");
                let maybe_server = local_server.upgrade();
                match maybe_server {
                    None => {
                        info!("Server is dead. So am I. Good bye!");
                        return;
                    }
                    Some(mserver) => {
                        log::info!("I'm gonna perform some job to save your data.");
                        let server = mserver.lock().expect("Mutex");
                        server.store(std::path::Path::new("server"));
                    }
                }
            }
        });
    }
}
