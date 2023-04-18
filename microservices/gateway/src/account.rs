use actix_web::{
    http::StatusCode,
    web::{self, Json},
    Responder,
};
use lunu::{
    account::{Approval, CustomerDesc, RetailerDesc, UpdateApproval},
    auth::Scope,
};

use crate::{tonic_code_to_status_code, User, ACCOUNT_CLIENT};

#[derive(serde::Deserialize)]
pub struct CustomerParams {
    account_id: String,
    first_name: String,
    last_name: String,
}

#[actix_web::post("/customer")]
pub async fn create_customer(user: User, params: Json<CustomerParams>) -> impl Responder {
    let User::Authenticated { account_id, scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and so you can't create a customer account."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    let CustomerParams {
        account_id: in_account_id,
        first_name,
        last_name,
    } = params.0;

    if !scopes.contains(&Scope::Admin) && account_id != in_account_id {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to create a customer account."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    let test = client
        .create_customer(CustomerDesc {
            account_id,
            first_name,
            last_name,
        })
        .await;
    match test {
        Ok(id) => (
            Json(serde_json::json!({
                "id": id.into_inner().id,
            })),
            StatusCode::CREATED,
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
pub struct RetailerParams {
    account_id: String,
}

#[actix_web::post("/retailer")]
pub async fn create_retailer(user: User, params: Json<RetailerParams>) -> impl Responder {
    let User::Authenticated { account_id, scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and so you can't create a retailer account."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    let RetailerParams {
        account_id: in_account_id,
    } = params.0;
    if !scopes.contains(&Scope::Admin) && account_id != in_account_id {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to create a retailer account."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    let test = client.create_retailer(RetailerDesc { account_id }).await;
    match test {
        Ok(id) => (
            Json(serde_json::json!({
                "id": id.into_inner().id,
            })),
            StatusCode::CREATED,
        ),
        Err(status) => (
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/customer/{customer_id}/approval")]
pub async fn update_approval_customer(
    user: User,
    path: web::Path<String>,
    params: Json<Approval>,
) -> impl Responder {
    let User::Authenticated { scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    if scopes.contains(&Scope::Admin) {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let customer_id = path.into_inner();

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    let test = client
        .update_approval_customer(UpdateApproval {
            id: customer_id,
            approval: params.0 as i32,
        })
        .await;
    match test {
        Ok(_) => (
            Json(serde_json::json!({
                "success": [],
            })),
            StatusCode::OK,
        ),
        Err(status) => (
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/retailer/{retailer_id}/approval")]
pub async fn update_approval_retailer(
    user: User,
    path: web::Path<String>,
    params: Json<Approval>,
) -> impl Responder {
    let User::Authenticated { scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    if scopes.contains(&Scope::Admin) {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let retailer_id = path.into_inner();

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    let test = client
        .update_approval_retailer(UpdateApproval {
            id: retailer_id,
            approval: params.0 as i32,
        })
        .await;
    match test {
        Ok(_) => (
            Json(serde_json::json!({
                "success": [],
            })),
            StatusCode::OK,
        ),
        Err(status) => (
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        ),
    }
}
