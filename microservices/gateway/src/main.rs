mod auth;
mod storage;

use std::time::Duration;

pub use auth::User;

use actix_web::{http, web, App, HttpServer};
use lunu::register_tonic_clients;
use tonic::transport::Channel;

register_tonic_clients! {
    (AUTH_CLIENT, lunu::auth::auth_client::AuthClient<Channel>, lunu::Microservice::Auth, "auth"),
    (STORAGE_CLIENT, lunu::storage::storage_client::StorageClient<Channel>, lunu::Microservice::Storage, "storage"),
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt().init();

    init_clients().await;

    tokio::spawn(async {
        let mut client = AUTH_CLIENT
            .get()
            .expect("AUTH_CLIENT used before it was initalized")
            .clone();

        loop {
            // Sleep for one day
            tokio::time::sleep(Duration::from_secs(86400)).await;
            if let Err(err) = client.cleanup_db(lunu::auth::Empty {}).await {
                tracing::error!("Error in cleaing up the auth db: {err}");
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api/v1/auth")
                    .service(auth::create_email_login_intent)
                    .service(auth::login_to_email_login_intent),
            )
            .service(
                web::scope("/api/v1/storage")
                    .service(storage::get_file)
                    .service(storage::put_file)
                    .service(storage::delete_file),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn tonic_code_to_status_code(code: tonic::Code) -> http::StatusCode {
    match code {
        tonic::Code::Ok => http::StatusCode::OK,
        tonic::Code::DeadlineExceeded => http::StatusCode::REQUEST_TIMEOUT,
        tonic::Code::InvalidArgument => http::StatusCode::BAD_REQUEST,
        tonic::Code::Cancelled
        | tonic::Code::DataLoss
        | tonic::Code::Internal
        | tonic::Code::Aborted
        | tonic::Code::ResourceExhausted
        | tonic::Code::AlreadyExists
        | tonic::Code::Unknown => http::StatusCode::INTERNAL_SERVER_ERROR,
        tonic::Code::NotFound => http::StatusCode::NOT_FOUND,
        tonic::Code::PermissionDenied => http::StatusCode::FORBIDDEN,
        tonic::Code::FailedPrecondition => http::StatusCode::PRECONDITION_FAILED,
        tonic::Code::OutOfRange => http::StatusCode::RANGE_NOT_SATISFIABLE,
        tonic::Code::Unimplemented => http::StatusCode::NOT_IMPLEMENTED,
        tonic::Code::Unavailable => http::StatusCode::NOT_FOUND,
        tonic::Code::Unauthenticated => http::StatusCode::UNAUTHORIZED,
    }
}
