use std::env;

use lunu::{
    account::{
        account_server::AccountServer, AccountDesc, AccountId, CustomerDesc, CustomerId,
        MerchantDesc, MerchantId,
    },
    diesel_async::{
        pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
        AsyncPgConnection,
    },
    dotenvy::dotenv,
    Microservice, MICROSERVICE_ADDRS,
};
use tonic::transport::Server;

struct Account {
    pool: Pool<AsyncPgConnection>,
}

#[tonic::async_trait]
impl lunu::account::account_server::Account for Account {
    async fn create_account(
        &self,
        request: tonic::Request<AccountDesc>,
    ) -> Result<tonic::Response<AccountId>, tonic::Status> {
        todo!()
    }

    async fn create_customer(
        &self,
        request: tonic::Request<CustomerDesc>,
    ) -> Result<tonic::Response<CustomerId>, tonic::Status> {
        todo!()
    }

    async fn create_merchant(
        &self,
        request: tonic::Request<MerchantDesc>,
    ) -> Result<tonic::Response<MerchantId>, tonic::Status> {
        todo!()
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
