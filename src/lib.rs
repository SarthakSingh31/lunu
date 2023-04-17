#[cfg(feature = "db")]
pub mod models;
#[cfg(feature = "db")]
pub mod schema;

use std::collections::HashMap;

#[cfg(feature = "db")]
pub use diesel;
#[cfg(feature = "db")]
pub use diesel_async;
pub use dotenvy;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Microservice {
    Auth,
    Account,
    Storage,
    Email,
}

lazy_static::lazy_static! {
    pub static ref MICROSERVICE_ADDRS: HashMap<Microservice, &'static str> = [
        (Microservice::Auth, "[::1]:50051"),
        (Microservice::Account, "[::1]:50052"),
        (Microservice::Storage, "[::1]:50053"),
        (Microservice::Email, "[::1]:50054"),
    ].into_iter().collect();
}

#[macro_export]
macro_rules! register_tonic_clients {
    ($(($name:ident, $client:ty, $varient:expr, $module:expr)),*$(,)?) => {
        $(register_tonic_clients!{static $name, $client})*

        async fn init_clients() {
            $(register_tonic_clients!{async $name, $client, $varient, $module})*
        }
    };
    (static $name:ident, $client:ty) => {
        static $name: tokio::sync::OnceCell<$client> = tokio::sync::OnceCell::const_new();
    };
    (async $name:ident, $client:ty, $varient:expr, $module:expr) => {
        let client = loop {
            match <$client>::connect(format!(
                "http://{}",
                lunu::MICROSERVICE_ADDRS[&$varient],
            ))
            .await
            {
                Ok(client) => break client,
                Err(_) => tokio::time::sleep(std::time::Duration::from_secs(1)).await,
            }
        };

        $name.set(client).expect(&format!("Failed to init {} client.", $module));

        println!("Connected to {} microservice!", $module);
    };
}

#[cfg(feature = "auth")]
pub mod auth {
    tonic::include_proto!("auth");
}

#[cfg(feature = "account")]
pub mod account {
    tonic::include_proto!("account");
}

#[cfg(feature = "storage")]
pub mod storage {
    tonic::include_proto!("storage");
}

#[cfg(feature = "email")]
pub mod email {
    tonic::include_proto!("email");
}
