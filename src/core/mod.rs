use once_cell::sync::Lazy;
use reqwest::Client;
use tokio::runtime::{Builder, Runtime};

static HTTP_CLIENT: Lazy<Client> = Lazy::new(Client::new);

pub static RT: Lazy<Runtime> = Lazy::new(|| {
    Builder::new_multi_thread()
        .worker_threads(2)
        .enable_io()
        .build()
        .expect("Failed to create multi-thread runtime")
});

pub mod helper;
pub mod installer;
