use actix_web::{
    http::StatusCode,
    web::{self, Json},
    Either, Responder,
};
use lunu::{
    account::{
        Approval, CustomerDesc, Id, LimitLevel, LimitPeriod, Limits, Money, PartnerDesc,
        PutPartnerFeeEntry, PutPartnerFees, PutRetailerFeeEntry, PutRetailerFees, RetailerDesc,
        RetailerPartner, Routing, SetApproval, SetLimit, SetLimitGlobal, SetMinPurchase,
        SetRouting,
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

#[actix_web::get("/customer/{customer_id}")]
pub async fn get_customer(user: User, path: web::Path<String>) -> impl Responder {
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

    match client.get_customer(Id { id: in_customer_id }).await {
        Ok(resp) => (Either::Left(Json(resp.into_inner())), StatusCode::OK),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
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

    match client
        .create_retailer(RetailerDesc {
            account_id: in_account_id,
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

#[actix_web::get("/retailer/{retailer_id}")]
pub async fn get_retailer(user: User, path: web::Path<String>) -> impl Responder {
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

    match client.get_customer(Id { id: in_retailer_id }).await {
        Ok(resp) => (Either::Left(Json(resp.into_inner())), StatusCode::OK),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/partner")]
pub async fn create_partner(user: User, params: Json<PartnerDesc>) -> impl Responder {
    let User::Authenticated { account_id, scopes , .. } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and so you can't create a retailer account."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    let PartnerDesc {
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

    match client
        .create_partner(PartnerDesc {
            account_id: in_account_id,
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

#[actix_web::get("/partner/{partner_id}")]
pub async fn get_partner(user: User, path: web::Path<String>) -> impl Responder {
    let User::Authenticated { partner_id, scopes , .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    let in_partner_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) && partner_id != Some(in_partner_id.clone()) {
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

    match client.get_partner(Id { id: in_partner_id }).await {
        Ok(resp) => (Either::Left(Json(resp.into_inner())), StatusCode::OK),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[derive(serde::Deserialize)]
pub struct RetailerPartnerParams {
    partner_id: String,
}

#[actix_web::post("/retailer/{retailer_id}/add_partner")]
pub async fn add_retailer_partner(
    user: User,
    path: web::Path<String>,
    params: Json<RetailerPartnerParams>,
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
        .add_retailer_partner(RetailerPartner {
            retailer_id,
            partner_id: params.0.partner_id,
        })
        .await
    {
        Ok(_resp) => (
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

#[actix_web::post("/retailer/{retailer_id}/remove_partner")]
pub async fn remove_retailer_partner(
    user: User,
    path: web::Path<String>,
    params: Json<RetailerPartnerParams>,
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
        .remove_retailer_partner(RetailerPartner {
            retailer_id,
            partner_id: params.0.partner_id,
        })
        .await
    {
        Ok(_resp) => (
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
        .get_approval_customer(Id { id: in_customer_id })
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
        .get_approval_retailer(Id { id: in_retailer_id })
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

    match client.get_customer_limits(Id { id: in_customer_id }).await {
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

    match client.get_retailer_limits(Id { id: in_retailer_id }).await {
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
        .get_min_purchase_value(Id { id: in_customer_id })
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

#[actix_web::get("/customer/{customer_id}/routing")]
pub async fn get_customer_routing(user: User, path: web::Path<String>) -> impl Responder {
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

    match client.get_customer_routing(Id { id: in_customer_id }).await {
        Ok(resp) => (Either::Left(Json(resp.into_inner())), StatusCode::OK),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/customer/{customer_id}/routing")]
pub async fn set_customer_routing(
    user: User,
    path: web::Path<String>,
    routing: Json<Routing>,
) -> impl Responder {
    let User::Authenticated {  scopes , .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    let customer_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) {
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
        .set_customer_routing(SetRouting {
            id: customer_id,
            routing: Some(routing.0),
        })
        .await
    {
        Ok(_resp) => (
            Either::Left(Json(serde_json::json!({
                "success": [],
            }))),
            StatusCode::OK,
        ),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::get("/retailer/{retailer_id}/routing")]
pub async fn get_retailer_routing(user: User, path: web::Path<String>) -> impl Responder {
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

    match client.get_retailer_routing(Id { id: in_retailer_id }).await {
        Ok(resp) => (Either::Left(Json(resp.into_inner())), StatusCode::OK),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/retailer/{retailer_id}/routing")]
pub async fn set_retailer_routing(
    user: User,
    path: web::Path<String>,
    routing: Json<Routing>,
) -> impl Responder {
    let User::Authenticated { scopes , .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    let retailer_id = path.into_inner();
    if !scopes.contains(&Scope::Admin) {
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
        .set_retailer_routing(SetRouting {
            id: retailer_id,
            routing: Some(routing.0),
        })
        .await
    {
        Ok(_resp) => (
            Either::Left(Json(serde_json::json!({
                "success": [],
            }))),
            StatusCode::OK,
        ),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::get("/global/routing")]
pub async fn get_global_routing(user: User) -> impl Responder {
    let User::Authenticated { .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client.get_global_routing(()).await {
        Ok(resp) => (Either::Left(Json(resp.into_inner())), StatusCode::OK),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/global/routing")]
pub async fn set_global_routing(user: User, routing: Json<Routing>) -> impl Responder {
    let User::Authenticated { scopes , .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    if !scopes.contains(&Scope::Admin) {
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

    match client.set_global_routing(routing.0).await {
        Ok(_resp) => (
            Either::Left(Json(serde_json::json!({
                "success": [],
            }))),
            StatusCode::OK,
        ),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::get("/retailer/{retailer_id}/fees")]
pub async fn get_retailer_fees(user: User, path: web::Path<String>) -> impl Responder {
    let User::Authenticated { .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    let retailer_id = path.into_inner();
    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client.get_retailer_fees(Id { id: retailer_id }).await {
        Ok(resp) => (Either::Left(Json(resp.into_inner())), StatusCode::OK),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/retailer/{retailer_id}/fees")]
pub async fn set_retailer_fees(
    user: User,
    path: web::Path<String>,
    fees: Json<Vec<PutRetailerFeeEntry>>,
) -> impl Responder {
    let User::Authenticated { scopes , .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    if !scopes.contains(&Scope::Admin) {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    }

    let retailer_id = path.into_inner();
    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .set_retailer_fees(PutRetailerFees {
            id: retailer_id,
            fees: fees.0,
        })
        .await
    {
        Ok(_resp) => (
            Either::Left(Json(serde_json::json!({
                "success": [],
            }))),
            StatusCode::OK,
        ),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::get("/partner/{partner_id}/fees")]
pub async fn get_partner_fees(user: User, path: web::Path<String>) -> impl Responder {
    let User::Authenticated { .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    let partner_id = path.into_inner();
    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client.get_partner_fees(Id { id: partner_id }).await {
        Ok(resp) => (Either::Left(Json(resp.into_inner())), StatusCode::OK),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}

#[actix_web::post("/partner/{partner_id}/fees")]
pub async fn set_partner_fees(
    user: User,
    path: web::Path<String>,
    fees: Json<Vec<PutPartnerFeeEntry>>,
) -> impl Responder {
    let User::Authenticated { scopes , .. } = user else {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You are not authenticated and can't access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    };

    if !scopes.contains(&Scope::Admin) {
        return (
            Either::Right(Json(serde_json::json!({
                "error": "You do not have permission to access this api."
            }))),
            StatusCode::UNAUTHORIZED,
        );
    }

    let partner_id = path.into_inner();
    let mut client = ACCOUNT_CLIENT
        .get()
        .expect("ACCOUNT_CLIENT used before it was initalized")
        .clone();

    match client
        .set_partner_fees(PutPartnerFees {
            id: partner_id,
            fees: fees.0,
        })
        .await
    {
        Ok(_resp) => (
            Either::Left(Json(serde_json::json!({
                "success": [],
            }))),
            StatusCode::OK,
        ),
        Err(status) => (
            Either::Right(Json(serde_json::json!({
                "error": status.message(),
            }))),
            tonic_code_to_status_code(status.code()),
        ),
    }
}
