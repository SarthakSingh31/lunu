use std::{collections::HashSet, fmt};

use actix_web::{http, web::Json, FromRequest, Responder, ResponseError};
use futures_util::future::LocalBoxFuture;
use lunu::auth::{AccountEmail, Scope, SessionToken};
use tonic::Status;

use crate::{tonic_code_to_status_code, AUTH_CLIENT};

pub enum User {
    Authenticated {
        account_id: String,
        scopes: HashSet<Scope>,
    },
    UnAuthenticated,
}

impl User {
    const SESSION_COOKIE: &'static str = "LUNU_SESSION";
    const DEFAULT_SCOPES: [Scope; 1] = [Scope::Public];

    pub fn is_account(&self, id: &str) -> bool {
        match self {
            User::Authenticated { account_id, .. } => account_id == id,
            User::UnAuthenticated => false,
        }
    }

    pub fn has_scopes(&self, required_scopes: impl IntoIterator<Item = Scope>) -> bool {
        match self {
            User::Authenticated { scopes, .. } => required_scopes
                .into_iter()
                .all(|scope| scopes.contains(&scope)),
            User::UnAuthenticated => required_scopes
                .into_iter()
                .all(|scope| Self::DEFAULT_SCOPES.contains(&scope)),
        }
    }

    pub fn is_authenticated(&self) -> bool {
        match self {
            User::Authenticated { .. } => true,
            User::UnAuthenticated => false,
        }
    }
}

impl FromRequest for User {
    type Error = AuthError;
    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        if let Some(session) = req
            .cookie(Self::SESSION_COOKIE)
            .as_ref()
            .map(|cookie| cookie.value())
        {
            let session = session.to_string();

            Box::pin(async move {
                let mut client = AUTH_CLIENT
                    .get()
                    .expect("AUTH_CLIENT used before it was initalized")
                    .clone();

                let account = client
                    .fetch_account(SessionToken { token: session })
                    .await
                    .map_err(|err| AuthError::FailedToFetchUser(err))?
                    .into_inner()
                    .account;

                if let Some(account) = account {
                    Ok(User::Authenticated {
                        scopes: account.scopes().collect(),
                        account_id: account.id,
                    })
                } else {
                    Ok(User::UnAuthenticated)
                }
            })
        } else {
            Box::pin(async { Ok(User::UnAuthenticated) })
        }
    }
}

#[derive(Debug)]
pub enum AuthError {
    FailedToFetchUser(Status),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::FailedToFetchUser(status) => f.write_fmt(format_args!(
                "Failed to fetch user from the auth microservice: {:?}",
                status.message()
            )),
        }
    }
}

impl ResponseError for AuthError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            AuthError::FailedToFetchUser(_) => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(serde::Deserialize)]
pub(super) struct EmailLoginIntent {
    email: String,
}

#[actix_web::post("/create_email_login_intent")]
pub(super) async fn create_email_login_intent(intent: Json<EmailLoginIntent>) -> impl Responder {
    let mut client = AUTH_CLIENT
        .get()
        .expect("AUTH_CLIENT used before it was initalized")
        .clone();

    let EmailLoginIntent { email } = intent.0;
    match client
        .create_email_login_intent(AccountEmail { email })
        .await
    {
        Ok(resp) => (
            Json(serde_json::json!({
                "token": resp.into_inner().token,
            })),
            http::StatusCode::OK,
        ),
        Err(status) => (
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[derive(serde::Deserialize)]
pub(super) struct EmailLoginParams {
    token: String,
    code: String,
}

#[actix_web::post("/login_to_email_login_intent")]
pub(super) async fn login_to_email_login_intent(params: Json<EmailLoginParams>) -> impl Responder {
    let mut client = AUTH_CLIENT
        .get()
        .expect("AUTH_CLIENT used before it was initalized")
        .clone();

    let EmailLoginParams { token, code } = params.0;
    match client
        .login_with_email_login(lunu::auth::EmailLoginParams { token, code })
        .await
    {
        Ok(resp) => (
            Json(serde_json::json!({
                "token": resp.into_inner().token,
            })),
            http::StatusCode::OK,
        ),
        Err(status) => (
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[derive(serde::Deserialize)]
pub(super) struct CreateNewPassLoginParams {
    email: String,
}

#[actix_web::post("/create_new_pass_login_intent")]
pub(super) async fn create_new_pass_login_intent(
    params: Json<CreateNewPassLoginParams>,
) -> impl Responder {
    let mut client = AUTH_CLIENT
        .get()
        .expect("AUTH_CLIENT used before it was initalized")
        .clone();

    let CreateNewPassLoginParams { email } = params.0;
    match client
        .create_new_pass_login_intent(lunu::auth::AccountEmail { email })
        .await
    {
        Ok(_resp) => (
            Json(serde_json::json!({
                "success": "The new passoword link was sent in the email",
            })),
            http::StatusCode::OK,
        ),
        Err(status) => (
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[derive(serde::Deserialize)]
pub(super) struct NewPassLoginParams {
    token: String,
    password: String,
}

#[actix_web::post("/login_with_new_pass_login")]
pub(super) async fn login_with_new_pass_login(params: Json<NewPassLoginParams>) -> impl Responder {
    let mut client = AUTH_CLIENT
        .get()
        .expect("AUTH_CLIENT used before it was initalized")
        .clone();

    let NewPassLoginParams { token, password } = params.0;
    match client
        .login_with_new_pass_login(lunu::auth::NewPassLoginParams { token, password })
        .await
    {
        Ok(resp) => (
            Json(serde_json::json!({
                "token": resp.into_inner().token,
            })),
            http::StatusCode::OK,
        ),
        Err(status) => (
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[derive(serde::Deserialize)]
pub(super) struct PasswordParams {
    email: String,
    password: String,
}

#[actix_web::post("/create_with_password")]
pub(super) async fn create_with_password(params: Json<PasswordParams>) -> impl Responder {
    let mut client = AUTH_CLIENT
        .get()
        .expect("AUTH_CLIENT used before it was initalized")
        .clone();

    let PasswordParams { email, password } = params.0;
    match client
        .create_with_password(lunu::auth::PasswordParams { email, password })
        .await
    {
        Ok(resp) => (
            Json(serde_json::json!({
                "token": resp.into_inner().token,
            })),
            http::StatusCode::OK,
        ),
        Err(status) => (
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/login_with_password")]
pub(super) async fn login_with_password(params: Json<PasswordParams>) -> impl Responder {
    let mut client = AUTH_CLIENT
        .get()
        .expect("AUTH_CLIENT used before it was initalized")
        .clone();

    let PasswordParams { email, password } = params.0;
    match client
        .login_with_password(lunu::auth::PasswordParams { email, password })
        .await
    {
        Ok(resp) => (
            Json(serde_json::json!({
                "token": resp.into_inner().token,
            })),
            http::StatusCode::OK,
        ),
        Err(status) => (
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        ),
    }
}
