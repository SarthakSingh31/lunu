mod account;
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
    (ACCOUNT_CLIENT, lunu::account::account_client::AccountClient<Channel>, lunu::Microservice::Account, "account"),
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
            if let Err(err) = client.cleanup_db(()).await {
                tracing::error!("Error in cleaing up the auth db: {err}");
            }
        }
    });

    HttpServer::new(move || {
        App::new()
            .service(
                web::scope("/api/v1/auth")
                    .service(auth::create_email_login_intent)
                    .service(auth::login_to_email_login_intent)
                    .service(auth::create_new_pass_login_intent)
                    .service(auth::login_with_new_pass_login)
                    .service(auth::create_with_password)
                    .service(auth::login_with_password),
            )
            .service(
                web::scope("/api/v1/storage")
                    .service(storage::get_file)
                    .service(storage::put_file)
                    .service(storage::delete_file),
            )
            .service(
                web::scope("/api/v1/account")
                    // Create & Get
                    .service(account::create_customer)
                    .service(account::get_customer)
                    .service(account::create_retailer)
                    .service(account::get_retailer)
                    .service(account::create_partner)
                    .service(account::get_partner)
                    // Retailer Partner
                    .service(account::add_retailer_partner)
                    .service(account::remove_retailer_partner)
                    // Approval
                    .service(account::get_approval_customer)
                    .service(account::set_approval_customer)
                    .service(account::get_approval_retailer)
                    .service(account::set_approval_retailer)
                    // Limits
                    .service(account::get_customer_limits)
                    .service(account::set_customer_limits)
                    .service(account::get_retailer_limits)
                    .service(account::set_retailer_limits)
                    .service(account::get_global_limits)
                    .service(account::set_global_limits)
                    // customer min purchase limit
                    .service(account::get_min_purchase_limit)
                    .service(account::set_min_purchase_limit)
                    // Routing
                    .service(account::get_customer_routing)
                    .service(account::set_customer_routing)
                    .service(account::get_retailer_routing)
                    .service(account::set_retailer_routing)
                    .service(account::get_global_routing)
                    .service(account::set_global_routing)
                    // Fees
                    .service(account::get_retailer_fees)
                    .service(account::set_retailer_fees)
                    .service(account::get_partner_fees)
                    .service(account::set_partner_fees),
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
