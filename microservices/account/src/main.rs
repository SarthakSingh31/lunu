use std::{env, str::FromStr};

use lunu::{
    account::{
        account_server::AccountServer, CustomerDesc, CustomerId, RetailerDesc, RetailerId,
        UpdateApproval,
    },
    diesel::{insert_into, update, ExpressionMethods},
    diesel_async::{
        pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
        AsyncPgConnection, RunQueryDsl,
    },
    dotenvy::dotenv,
    models, schema, Microservice, MICROSERVICE_ADDRS,
};
use time::OffsetDateTime;
use tonic::transport::Server;
use uuid::Uuid;

struct Account {
    pool: Pool<AsyncPgConnection>,
}

#[tonic::async_trait]
impl lunu::account::account_server::Account for Account {
    async fn create_customer(
        &self,
        request: tonic::Request<CustomerDesc>,
    ) -> Result<tonic::Response<CustomerId>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        let CustomerDesc {
            account_id,
            first_name,
            last_name,
        } = request.into_inner();
        let account_id =
            Uuid::from_str(&account_id).map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::customers::dsl as c_dsl;

        let id = Uuid::new_v4();
        insert_into(c_dsl::customers)
            .values(models::Customer {
                id,
                first_name,
                last_name,
                account_id,
            })
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(CustomerId { id: id.to_string() }))
    }

    async fn create_retailer(
        &self,
        request: tonic::Request<RetailerDesc>,
    ) -> Result<tonic::Response<RetailerId>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        let RetailerDesc { account_id } = request.into_inner();
        let account_id =
            Uuid::from_str(&account_id).map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::retailers::dsl as r_dsl;

        let id = Uuid::new_v4();
        insert_into(r_dsl::retailers)
            .values(models::Retailer { id, account_id })
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(RetailerId { id: id.to_string() }))
    }

    async fn update_approval_customer(
        &self,
        request: tonic::Request<UpdateApproval>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let u_approval: UpdateApproval = request.into_inner();
        let id = Uuid::from_str(&u_approval.id).map_err(|_| AccountError::MalformedAccountToken)?;
        let approval: models::Approval = u_approval.approval().into();

        use schema::customers::dsl as c_dsl;

        let approved_at = match approval {
            models::Approval::Approved => Some(OffsetDateTime::now_utc()),
            models::Approval::Rejected => None,
            models::Approval::OnHold => None,
        };

        update(c_dsl::customers)
            .filter(c_dsl::id.eq(id))
            .set((
                c_dsl::approved.eq(approval),
                c_dsl::approved_at.eq(approved_at),
            ))
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(()))
    }

    async fn update_approval_retailer(
        &self,
        request: tonic::Request<UpdateApproval>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let u_approval: UpdateApproval = request.into_inner();
        let id = Uuid::from_str(&u_approval.id).map_err(|_| AccountError::MalformedAccountToken)?;
        let approval: models::Approval = u_approval.approval().into();

        use schema::retailers::dsl as r_dsl;

        let approved_at = match approval {
            models::Approval::Approved => Some(OffsetDateTime::now_utc()),
            models::Approval::Rejected => None,
            models::Approval::OnHold => None,
        };

        update(r_dsl::retailers)
            .filter(r_dsl::id.eq(id))
            .set((
                r_dsl::approved.eq(approval),
                r_dsl::approved_at.eq(approved_at),
            ))
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(()))
    }
}

enum AccountError {
    MalformedAccountToken,
    QueryFailed(String),
    PoolConnectionFailed,
}

impl From<AccountError> for tonic::Status {
    fn from(value: AccountError) -> Self {
        match value {
            AccountError::MalformedAccountToken => {
                tonic::Status::invalid_argument("Malformed session token")
            }
            AccountError::QueryFailed(s) => tonic::Status::internal(format!("Query Failed: {s}")),
            AccountError::PoolConnectionFailed => {
                tonic::Status::internal("Failed to connect to the internal pool")
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
    let account = Account {
        pool: Pool::builder().build(config).await?,
    };

    let addr = MICROSERVICE_ADDRS[&Microservice::Account].parse()?;
    Server::builder()
        .add_service(AccountServer::new(account))
        .serve(addr)
        .await?;

    Ok(())
}
