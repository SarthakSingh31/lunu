use std::{env, str::FromStr};

use bigdecimal::{num_bigint::BigInt, BigDecimal};
use lunu::{
    account::{
        account_server::AccountServer, Approval, CustomerDesc, CustomerId, GetApproval,
        InnerLimits, Limits, Money, RetailerDesc, RetailerId, SetApproval, SetLimit,
        SetLimitGlobal, SetMinPurchase,
    },
    diesel::{insert_into, update, ExpressionMethods, QueryDsl},
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

    async fn get_approval_customer(
        &self,
        request: tonic::Request<CustomerId>,
    ) -> Result<tonic::Response<GetApproval>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let id = Uuid::from_str(&request.into_inner().id)
            .map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::customers::dsl as c_dsl;

        let approval: Approval = c_dsl::customers
            .filter(c_dsl::id.eq(id))
            .select(c_dsl::approved)
            .load::<models::Approval>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .pop()
            .ok_or(AccountError::CustomerNotFound)?
            .into();

        Ok(tonic::Response::new(GetApproval {
            approval: approval as i32,
        }))
    }

    async fn set_approval_customer(
        &self,
        request: tonic::Request<SetApproval>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let u_approval: SetApproval = request.into_inner();
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

    async fn get_approval_retailer(
        &self,
        request: tonic::Request<RetailerId>,
    ) -> Result<tonic::Response<GetApproval>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let id = Uuid::from_str(&request.into_inner().id)
            .map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::retailers::dsl as r_dsl;

        let approval: Approval = r_dsl::retailers
            .filter(r_dsl::id.eq(id))
            .select(r_dsl::approved)
            .load::<models::Approval>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .pop()
            .ok_or(AccountError::RetailerNotFound)?
            .into();

        Ok(tonic::Response::new(GetApproval {
            approval: approval as i32,
        }))
    }

    async fn set_approval_retailer(
        &self,
        request: tonic::Request<SetApproval>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let u_approval: SetApproval = request.into_inner();
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

    async fn get_customer_limits(
        &self,
        request: tonic::Request<CustomerId>,
    ) -> Result<tonic::Response<InnerLimits>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let CustomerId { id } = request.into_inner();
        let id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::customer_limits::dsl as cl_dsl;

        let limits = cl_dsl::customer_limits
            .select((
                cl_dsl::period,
                cl_dsl::level,
                cl_dsl::amount,
                cl_dsl::currency,
            ))
            .filter(cl_dsl::customer_id.eq(id))
            .load::<(models::LimitPeriod, models::LimitLevel, BigDecimal, String)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;
        let limits = Limits(
            limits
                .into_iter()
                .map(|(period, level, amount, currency)| {
                    let (digits, exponent) = amount.into_bigint_and_exponent();
                    let money = Money {
                        currency_code: currency,
                        digits: digits.to_signed_bytes_le(),
                        exponent,
                    };
                    ((period.into(), level.into()), money)
                })
                .collect(),
        );

        Ok(tonic::Response::new(limits.into()))
    }

    async fn set_customer_limit(
        &self,
        request: tonic::Request<SetLimit>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let limit = request.into_inner();

        let period = limit.period().into();
        let level = limit.level().into();
        let id = Uuid::from_str(&limit.id).map_err(|_| AccountError::MalformedAccountToken)?;
        let Some(amount) = limit.amount else {
            return Err(AccountError::MissingAmount.into());
        };

        use schema::customer_limits::dsl as cl_dsl;

        insert_into(cl_dsl::customer_limits)
            .values(models::CustomerLimit {
                period,
                level,
                amount: BigDecimal::new(
                    BigInt::from_signed_bytes_le(&amount.digits),
                    amount.exponent,
                ),
                currency: amount.currency_code.as_str(),
                customer_id: id,
            })
            .on_conflict((cl_dsl::period, cl_dsl::level, cl_dsl::customer_id))
            .do_update()
            .set((
                cl_dsl::amount.eq(BigDecimal::new(
                    BigInt::from_signed_bytes_le(&amount.digits),
                    amount.exponent,
                )),
                cl_dsl::currency.eq(amount.currency_code.as_str()),
            ))
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(()))
    }

    async fn get_retailer_limits(
        &self,
        request: tonic::Request<RetailerId>,
    ) -> Result<tonic::Response<InnerLimits>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let RetailerId { id } = request.into_inner();
        let id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::retailer_limits::dsl as rl_dsl;

        let limits = rl_dsl::retailer_limits
            .select((
                rl_dsl::period,
                rl_dsl::level,
                rl_dsl::amount,
                rl_dsl::currency,
            ))
            .filter(rl_dsl::retailer_id.eq(id))
            .load::<(models::LimitPeriod, models::LimitLevel, BigDecimal, String)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;
        let limits = Limits(
            limits
                .into_iter()
                .map(|(period, level, amount, currency)| {
                    let (digits, exponent) = amount.into_bigint_and_exponent();
                    let money = Money {
                        currency_code: currency,
                        digits: digits.to_signed_bytes_le(),
                        exponent,
                    };
                    ((period.into(), level.into()), money)
                })
                .collect(),
        );

        Ok(tonic::Response::new(limits.into()))
    }

    async fn set_retailer_limit(
        &self,
        request: tonic::Request<SetLimit>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let limit = request.into_inner();

        let period = limit.period().into();
        let level = limit.level().into();
        let id = Uuid::from_str(&limit.id).map_err(|_| AccountError::MalformedAccountToken)?;
        let Some(amount) = limit.amount else {
            return Err(AccountError::MissingAmount.into());
        };

        use schema::retailer_limits::dsl as rl_dsl;

        insert_into(rl_dsl::retailer_limits)
            .values(models::RetailerLimit {
                period,
                level,
                amount: BigDecimal::new(
                    BigInt::from_signed_bytes_le(&amount.digits),
                    amount.exponent,
                ),
                currency: amount.currency_code.as_str(),
                retailer_id: id,
            })
            .on_conflict((rl_dsl::period, rl_dsl::level, rl_dsl::retailer_id))
            .do_update()
            .set((
                rl_dsl::amount.eq(BigDecimal::new(
                    BigInt::from_signed_bytes_le(&amount.digits),
                    amount.exponent,
                )),
                rl_dsl::currency.eq(amount.currency_code.as_str()),
            ))
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(()))
    }

    async fn get_global_limits(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<InnerLimits>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::global_limits::dsl as gl_dsl;

        let limits = gl_dsl::global_limits
            .load::<(models::LimitPeriod, models::LimitLevel, BigDecimal, String)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;
        let limits = Limits(
            limits
                .into_iter()
                .map(|(period, level, amount, currency)| {
                    ((period.into(), level.into()), (currency, amount).into())
                })
                .collect(),
        );

        Ok(tonic::Response::new(limits.into()))
    }

    async fn set_global_limit(
        &self,
        request: tonic::Request<SetLimitGlobal>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let limit = request.into_inner();

        let period = limit.period().into();
        let level = limit.level().into();
        let Some(amount) = limit.amount else {
            return Err(AccountError::MissingAmount.into());
        };

        use schema::global_limits::dsl as gl_dsl;

        let (currency, amount) = amount.into();

        insert_into(gl_dsl::global_limits)
            .values(models::GlobalLimit {
                period,
                level,
                amount: amount.clone(),
                currency: &currency,
            })
            .on_conflict((gl_dsl::period, gl_dsl::level))
            .do_update()
            .set((
                gl_dsl::amount.eq(amount),
                gl_dsl::currency.eq(currency.as_str()),
            ))
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(()))
    }

    async fn get_min_purchase_value(
        &self,
        request: tonic::Request<CustomerId>,
    ) -> Result<tonic::Response<Money>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let id = Uuid::from_str(&request.into_inner().id)
            .map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::customers::dsl as c_dsl;

        let money = c_dsl::customers
            .filter(c_dsl::id.eq(id))
            .select((c_dsl::min_purchase_currency, c_dsl::min_purchase_amount))
            .load::<(String, BigDecimal)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .pop()
            .ok_or(AccountError::CustomerNotFound)?
            .into();

        Ok(tonic::Response::new(money))
    }

    async fn set_min_purchase_value(
        &self,
        request: tonic::Request<SetMinPurchase>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let SetMinPurchase {
            customer_id,
            amount,
        } = request.into_inner();
        let id = Uuid::from_str(&customer_id).map_err(|_| AccountError::MalformedAccountToken)?;
        let (currency, amount): (String, BigDecimal) =
            amount.ok_or(AccountError::MissingAmount)?.into();

        use schema::customers::dsl as c_dsl;

        update(c_dsl::customers)
            .filter(c_dsl::id.eq(id))
            .set((
                c_dsl::min_purchase_currency.eq(currency),
                c_dsl::min_purchase_amount.eq(amount),
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
    MissingAmount,
    CustomerNotFound,
    RetailerNotFound,
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
            AccountError::MissingAmount => {
                tonic::Status::invalid_argument("Missing amount from request")
            }
            AccountError::CustomerNotFound => {
                tonic::Status::invalid_argument("Customer with the supplied id was not found")
            }
            AccountError::RetailerNotFound => {
                tonic::Status::invalid_argument("Retailer with the supplied id was not found")
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
