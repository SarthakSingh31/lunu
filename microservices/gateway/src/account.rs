use actix_web::{
    http::StatusCode,
    web::{self, Json},
    Either, Responder,
};
use lunu::{
    account::{
        Approval, CustomerDesc, CustomerId, LimitLevel, LimitPeriod, Limits, Money, RetailerDesc,
        RetailerId, SetApproval, SetLimit, SetLimitGlobal, SetMinPurchase,
    },
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

    match client
        .create_customer(CustomerDesc {
            account_id,
            first_name,
            last_name,
        })
        .await
    {
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

    match client.create_retailer(RetailerDesc { account_id }).await {
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

#[actix_web::get("/customer/{customer_id}/approval")]
pub async fn get_approval_customer(user: User, path: web::Path<String>) -> impl Responder {
    let User::Authenticated { customer_id, scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    let in_customer_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) && customer_id != Some(in_customer_id.clone()) {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .get_approval_customer(CustomerId { id: in_customer_id })
        .await
    {
        Ok(resp) => (
            Json(serde_json::json!({
                "approval": resp.into_inner().approval(),
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

#[actix_web::post("/customer/{customer_id}/approval")]
pub async fn set_approval_customer(
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

    if !scopes.contains(&Scope::Admin) {
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

    match client
        .set_approval_customer(SetApproval {
            id: customer_id,
            approval: params.0 as i32,
        })
        .await
    {
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

#[actix_web::get("/retailer/{retailer_id}/approval")]
pub async fn get_approval_retailer(user: User, path: web::Path<String>) -> impl Responder {
    let User::Authenticated { retailer_id, scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    let in_retailer_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) && retailer_id != Some(in_retailer_id.clone()) {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .get_approval_retailer(RetailerId { id: in_retailer_id })
        .await
    {
        Ok(resp) => (
            Json(serde_json::json!({
                "approval": resp.into_inner().approval(),
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
pub async fn set_approval_retailer(
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

    if !scopes.contains(&Scope::Admin) {
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

    match client
        .set_approval_retailer(SetApproval {
            id: retailer_id,
            approval: params.0 as i32,
        })
        .await
    {
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

#[actix_web::get("/customer/{customer_id}/limits")]
pub async fn get_customer_limits(user: User, path: web::Path<String>) -> impl Responder {
    let User::Authenticated { customer_id, scopes , .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    let in_customer_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) && customer_id != Some(in_customer_id.clone()) {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .get_customer_limits(CustomerId { id: in_customer_id })
        .await
    {
        Ok(resp) => {
            let resp: Limits = resp.into_inner().into();
            (Either::Left(Json(resp)), StatusCode::OK)
        }
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[derive(serde::Deserialize)]
pub struct SetLimitParams {
    period: LimitPeriod,
    level: LimitLevel,
    amount: Money,
}

#[actix_web::post("/customer/{customer_id}/limits")]
pub async fn set_customer_limits(
    user: User,
    path: web::Path<String>,
    param: Json<SetLimitParams>,
) -> impl Responder {
    let User::Authenticated { scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    let customer_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .set_customer_limit(SetLimit {
            period: param.period as i32,
            level: param.level as i32,
            id: customer_id,
            amount: Some(param.0.amount),
        })
        .await
    {
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

#[actix_web::get("/retailer/{retailer_id}/limits")]
pub async fn get_retailer_limits(user: User, path: web::Path<String>) -> impl Responder {
    let User::Authenticated { retailer_id, scopes , .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    let in_retailer_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) && retailer_id != Some(in_retailer_id.clone()) {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .get_retailer_limits(RetailerId { id: in_retailer_id })
        .await
    {
        Ok(resp) => {
            let resp: Limits = resp.into_inner().into();
            (Either::Left(Json(resp)), StatusCode::OK)
        }
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/retailer/{retailer_id}/limits")]
pub async fn set_retailer_limits(
    user: User,
    path: web::Path<String>,
    param: Json<SetLimitParams>,
) -> impl Responder {
    let User::Authenticated { scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    let retailer_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .set_retailer_limit(SetLimit {
            period: param.period as i32,
            level: param.level as i32,
            id: retailer_id,
            amount: Some(param.0.amount),
        })
        .await
    {
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

#[actix_web::get("/global/limits")]
pub async fn get_global_limits() -> impl Responder {
    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client.get_global_limits(()).await {
        Ok(resp) => {
            let resp: Limits = resp.into_inner().into();
            (Either::Left(Json(resp)), StatusCode::OK)
        }
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/global/limits")]
pub async fn set_global_limits(user: User, param: Json<SetLimitParams>) -> impl Responder {
    let User::Authenticated { scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    if !scopes.contains(&Scope::Admin) {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .set_global_limit(SetLimitGlobal {
            period: param.period as i32,
            level: param.level as i32,
            amount: Some(param.0.amount),
        })
        .await
    {
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

#[actix_web::get("/customer/{customer_id}/min_purchase_limit")]
pub async fn get_min_purchase_limit(user: User, path: web::Path<String>) -> impl Responder {
    let User::Authenticated { customer_id, scopes , .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    let in_customer_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) && customer_id != Some(in_customer_id.clone()) {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .get_min_purchase_value(CustomerId { id: in_customer_id })
        .await
    {
        Ok(resp) => (Either::Left(Json(resp.into_inner())), StatusCode::OK),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/customer/{customer_id}/min_purchase_limit")]
pub async fn set_min_purchase_limit(
    user: User,
    path: web::Path<String>,
    param: Json<Money>,
) -> impl Responder {
    let User::Authenticated { scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    let customer_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .set_min_purchase_value(SetMinPurchase {
            customer_id,
            amount: Some(param.0),
        })
        .await
    {
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
