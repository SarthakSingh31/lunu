use actix_web::{
    dev::Response,
    http::{
        header::{self, HeaderValue},
        StatusCode,
    },
    web::{self, Json},
    Either, Responder,
};
use lunu::{
    auth::Scope,
    storage::{File, FileId},
};

use crate::{tonic_code_to_status_code, User, STORAGE_CLIENT};

#[actix_web::get("/{account_id}/{name}")]
pub async fn get_file(user: User, path: web::Path<(String, String)>) -> impl Responder {
    let (in_account_id, name) = path.into_inner();

    let User::Authenticated { account_id, scopes } = user else {
        return Either::Right((
            Json(serde_json::json!({
                "error": "You are not authenticated and can't view files."
            })),
            StatusCode::UNAUTHORIZED,
        ));
    };

    if !scopes.contains(&Scope::Admin) && account_id == in_account_id {
        return Either::Right((
            Json(serde_json::json!({
                "error": "You do not have permission to view this file."
            })),
            StatusCode::UNAUTHORIZED,
        ));
    }

    let mine = mime_guess::from_path(&name);

    let mut client = STORAGE_CLIENT
        .get()
        .expect("STORAGE_CLIENT used before it was initalized")
        .clone();

    match client
        .get(FileId {
            account_id: in_account_id,
            name,
        })
        .await
    {
        Ok(data) => {
            let data = data.into_inner();

            if let Some(data) = data.data {
                let mut resp = Response::new(StatusCode::OK)
                    .set_body(data)
                    .map_into_boxed_body();

                let headers = resp.headers_mut();
                if let Some(mime) = mine.first() {
                    headers.append(
                        header::CONTENT_TYPE,
                        HeaderValue::from_bytes(mime.essence_str().as_bytes()).unwrap(),
                    );
                } else {
                    headers.append(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("application/octet-stream"),
                    );
                }

                Either::Left(resp)
            } else {
                Either::Right((
                    Json(serde_json::json!({
                        "error": "Failed to find the specfied file."
                    })),
                    StatusCode::NOT_FOUND,
                ))
            }
        }
        Err(status) => Either::Right((
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        )),
    }
}

// TODO: Add some way to make it only possible for the user to put files that have been approved.
// They should not be able to put files at anytime. They should only be able to put files when the
// frontend requests it.
#[actix_web::put("/{account_id}/{name}")]
pub async fn put_file(
    user: User,
    path: web::Path<(String, String)>,
    bytes: web::Bytes,
) -> impl Responder {
    let (in_account_id, name) = path.into_inner();

    let User::Authenticated { account_id, scopes } = user else {
        return Either::Right((
            Json(serde_json::json!({
                "error": "You are not authenticated and can't write files."
            })),
            StatusCode::UNAUTHORIZED,
        ));
    };

    if !scopes.contains(&Scope::Admin) && account_id == in_account_id {
        return Either::Right((
            Json(serde_json::json!({
                "error": "You do not have permission to write this file."
            })),
            StatusCode::UNAUTHORIZED,
        ));
    }

    let mine = mime_guess::from_path(&name);

    let mut client = STORAGE_CLIENT
        .get()
        .expect("STORAGE_CLIENT used before it was initalized")
        .clone();

    match client
        .put(File {
            id: Some(FileId { account_id, name }),
            data: bytes.into_iter().collect(),
        })
        .await
    {
        Ok(data) => {
            let data = data.into_inner();

            if let Some(data) = data.data {
                let mut resp = Response::new(StatusCode::OK)
                    .set_body(data)
                    .map_into_boxed_body();

                let headers = resp.headers_mut();
                if let Some(mime) = mine.first() {
                    headers.append(
                        header::CONTENT_TYPE,
                        HeaderValue::from_bytes(mime.essence_str().as_bytes()).unwrap(),
                    );
                } else {
                    headers.append(
                        header::CONTENT_TYPE,
                        HeaderValue::from_static("application/octet-stream"),
                    );
                }

                Either::Left(resp)
            } else {
                Either::Right((Json(serde_json::json!({})), StatusCode::OK))
            }
        }
        Err(status) => Either::Right((
            Json(serde_json::json!({
                "error": status.message(),
            })),
            tonic_code_to_status_code(status.code()),
        )),
    }
}

// TODO: It is unclear when a user might need to delete the files
#[actix_web::delete("/{account_id}/{name}")]
pub async fn delete_file(user: User, path: web::Path<(String, String)>) -> impl Responder {
    let (in_account_id, name) = path.into_inner();

    let User::Authenticated { account_id, scopes } = user else {
        return (
            Json(serde_json::json!({
                "error": "You are not authenticated and can't delete files."
            })),
            StatusCode::UNAUTHORIZED,
        );
    };

    if !scopes.contains(&Scope::Admin) && account_id == in_account_id {
        return (
            Json(serde_json::json!({
                "error": "You do not have permission to delete this file."
            })),
            StatusCode::UNAUTHORIZED,
        );
    }

    let mut client = STORAGE_CLIENT
        .get()
        .expect("STORAGE_CLIENT used before it was initalized")
        .clone();

    match client.delete(FileId { account_id, name }).await {
        Ok(_data) => (
            Json(serde_json::json!({
                "success": "File Deleted.",
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
