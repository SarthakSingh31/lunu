mod helpers;

use std::{env, str::FromStr};

use bigdecimal::{num_bigint::BigInt, BigDecimal};
use helpers::routing::RoutingTable;
use lunu::{
    account::{
        account_server::AccountServer, Approval, CustomerData, CustomerDesc, GetApproval, Id,
        InnerLimits, KycLevel, Limits, Money, PartnerData, PartnerDesc, PartnerFees,
        PutPartnerFees, PutRetailerFees, RetailerData, RetailerDesc, RetailerFees, RetailerPartner,
        Routing, SetApproval, SetLimit, SetLimitGlobal, SetMinPurchase, SetRouting,
    },
    diesel::{delete, insert_into, update, BoolExpressionMethods, ExpressionMethods, QueryDsl},
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
    ) -> Result<tonic::Response<Id>, tonic::Status> {
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

        Ok(tonic::Response::new(Id { id: id.to_string() }))
    }

    async fn get_customer(
        &self,
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<CustomerData>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        let Id { id } = request.into_inner();
        let customer_id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::customers::dsl as c_dsl;

        let data = c_dsl::customers
            .filter(c_dsl::id.eq(customer_id))
            .select((
                c_dsl::first_name,
                c_dsl::last_name,
                c_dsl::kyc_level,
                c_dsl::approved_at,
                c_dsl::approved,
                c_dsl::residence_address,
                c_dsl::country_of_residence,
            ))
            .first::<(
                String,
                String,
                models::KycLevel,
                Option<OffsetDateTime>,
                Option<models::Approval>,
                Option<String>,
                Option<String>,
            )>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(CustomerData {
            first_name: data.0,
            last_name: data.1,
            kyc_level: KycLevel::from(data.2) as i32,
            approved_at: data.3.map(|time| time.to_string()),
            approved: data.4.map(|approval| Approval::from(approval) as i32),
            residence_address: data.5,
            country_of_residence: data.6,
        }))
    }

    async fn create_retailer(
        &self,
        request: tonic::Request<RetailerDesc>,
    ) -> Result<tonic::Response<Id>, tonic::Status> {
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

        Ok(tonic::Response::new(Id { id: id.to_string() }))
    }

    async fn get_retailer(
        &self,
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<RetailerData>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        let Id { id } = request.into_inner();
        let retailer_id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::retailers::dsl as r_dsl;

        let data = r_dsl::retailers
            .filter(r_dsl::id.eq(retailer_id))
            .select((
                r_dsl::addr_line_1,
                r_dsl::addr_line_2,
                r_dsl::country,
                r_dsl::approved_at,
                r_dsl::approved,
            ))
            .first::<(
                Option<String>,
                Option<String>,
                Option<String>,
                Option<OffsetDateTime>,
                Option<models::Approval>,
            )>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        use schema::retailer_partners::dsl as rp_dsl;

        let partners = rp_dsl::retailer_partners
            .filter(rp_dsl::retailer_id.eq(retailer_id))
            .select(rp_dsl::partner_id)
            .load::<Uuid>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(RetailerData {
            addr_line_1: data.0,
            addr_line_2: data.1,
            country: data.2,
            approved_at: data.3.map(|time| time.to_string()),
            approved: data.4.map(|approval| Approval::from(approval) as i32),
            partners: partners
                .into_iter()
                .map(|partner| partner.to_string())
                .collect(),
        }))
    }

    async fn create_partner(
        &self,
        request: tonic::Request<PartnerDesc>,
    ) -> Result<tonic::Response<Id>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        let PartnerDesc { account_id } = request.into_inner();
        let account_id =
            Uuid::from_str(&account_id).map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::partners::dsl as p_dsl;

        let id = Uuid::new_v4();
        insert_into(p_dsl::partners)
            .values(models::Partner { id, account_id })
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(Id { id: id.to_string() }))
    }

    async fn get_partner(
        &self,
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<PartnerData>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        let Id { id } = request.into_inner();
        let partner_id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::partners::dsl as p_dsl;

        let data = p_dsl::partners
            .filter(p_dsl::id.eq(partner_id))
            .select((p_dsl::approved_at, p_dsl::approved))
            .first::<(Option<OffsetDateTime>, Option<models::Approval>)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(PartnerData {
            approved_at: data.0.map(|time| time.to_string()),
            approved: data.1.map(|approval| Approval::from(approval) as i32),
        }))
    }

    async fn add_retailer_partner(
        &self,
        request: tonic::Request<RetailerPartner>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        let RetailerPartner {
            retailer_id,
            partner_id,
        } = request.into_inner();

        let retailer_id =
            Uuid::from_str(&retailer_id).map_err(|_| AccountError::MalformedAccountToken)?;
        let partner_id =
            Uuid::from_str(&partner_id).map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::retailer_partners::dsl as rp_dsl;

        insert_into(rp_dsl::retailer_partners)
            .values((
                rp_dsl::retailer_id.eq(retailer_id),
                rp_dsl::partner_id.eq(partner_id),
            ))
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(()))
    }

    async fn remove_retailer_partner(
        &self,
        request: tonic::Request<RetailerPartner>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        let RetailerPartner {
            retailer_id,
            partner_id,
        } = request.into_inner();

        let retailer_id =
            Uuid::from_str(&retailer_id).map_err(|_| AccountError::MalformedAccountToken)?;
        let partner_id =
            Uuid::from_str(&partner_id).map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::retailer_partners::dsl as rp_dsl;

        delete(rp_dsl::retailer_partners)
            .filter(
                rp_dsl::retailer_id
                    .eq(retailer_id)
                    .and(rp_dsl::partner_id.eq(partner_id)),
            )
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(()))
    }

    async fn get_approval_customer(
        &self,
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<GetApproval>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let id = Uuid::from_str(&request.into_inner().id)
            .map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::customers::dsl as c_dsl;

        let approval = c_dsl::customers
            .filter(c_dsl::id.eq(id))
            .select(c_dsl::approved)
            .load::<Option<models::Approval>>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .pop()
            .ok_or(AccountError::CustomerNotFound)?;

        Ok(tonic::Response::new(GetApproval {
            approval: approval.map(|approval| Approval::from(approval) as i32),
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
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<GetApproval>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let id = Uuid::from_str(&request.into_inner().id)
            .map_err(|_| AccountError::MalformedAccountToken)?;

        use schema::retailers::dsl as r_dsl;

        let approval = r_dsl::retailers
            .filter(r_dsl::id.eq(id))
            .select(r_dsl::approved)
            .load::<Option<models::Approval>>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .pop()
            .ok_or(AccountError::RetailerNotFound)?;

        Ok(tonic::Response::new(GetApproval {
            approval: approval.map(|approval| Approval::from(approval) as i32),
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
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<InnerLimits>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let Id { id } = request.into_inner();
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
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<InnerLimits>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;
        let Id { id } = request.into_inner();
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
        request: tonic::Request<Id>,
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

    async fn get_customer_routing(
        &self,
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<Routing>, tonic::Status> {
        let Id { id } = request.into_inner();
        let id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;

        Ok(tonic::Response::new(
            helpers::routing::CustomerRouting::get(&self.pool, id).await?,
        ))
    }

    async fn set_customer_routing(
        &self,
        request: tonic::Request<SetRouting>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let SetRouting { id, routing } = request.into_inner();
        let id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;
        let Some(routing) = routing else {
            return Err(AccountError::MissingRoutingData.into());
        };

        Ok(tonic::Response::new(
            helpers::routing::CustomerRouting::set(&self.pool, id, routing).await?,
        ))
    }

    async fn get_retailer_routing(
        &self,
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<Routing>, tonic::Status> {
        let Id { id } = request.into_inner();
        let id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;

        Ok(tonic::Response::new(
            helpers::routing::RetailerRouting::get(&self.pool, id).await?,
        ))
    }

    async fn set_retailer_routing(
        &self,
        request: tonic::Request<SetRouting>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let SetRouting { id, routing } = request.into_inner();
        let id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;
        let Some(routing) = routing else {
            return Err(AccountError::MissingRoutingData.into());
        };

        Ok(tonic::Response::new(
            helpers::routing::RetailerRouting::set(&self.pool, id, routing).await?,
        ))
    }

    async fn get_global_routing(
        &self,
        _request: tonic::Request<()>,
    ) -> Result<tonic::Response<Routing>, tonic::Status> {
        Ok(tonic::Response::new(
            helpers::routing::GlobalRouting::get(&self.pool, ()).await?,
        ))
    }

    async fn set_global_routing(
        &self,
        request: tonic::Request<Routing>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let routing = request.into_inner();

        Ok(tonic::Response::new(
            helpers::routing::GlobalRouting::set(&self.pool, (), routing).await?,
        ))
    }

    async fn get_retailer_fees(
        &self,
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<RetailerFees>, tonic::Status> {
        let id = Uuid::from_str(&request.into_inner().id)
            .map_err(|_| AccountError::MalformedAccountToken)?;
        let fees = helpers::fees::RetailerFees(&self.pool);

        Ok(tonic::Response::new(fees.get(id).await?))
    }

    async fn set_retailer_fees(
        &self,
        request: tonic::Request<PutRetailerFees>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let PutRetailerFees {
            id,
            fees: retailer_fees,
        } = request.into_inner();
        let id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;
        let fees = helpers::fees::RetailerFees(&self.pool);

        fees.set(id, retailer_fees).await?;

        Ok(tonic::Response::new(()))
    }

    async fn get_partner_fees(
        &self,
        request: tonic::Request<Id>,
    ) -> Result<tonic::Response<PartnerFees>, tonic::Status> {
        let id = Uuid::from_str(&request.into_inner().id)
            .map_err(|_| AccountError::MalformedAccountToken)?;
        let fees = helpers::fees::PartnerFees(&self.pool);

        Ok(tonic::Response::new(fees.get(id).await?))
    }

    async fn set_partner_fees(
        &self,
        request: tonic::Request<PutPartnerFees>,
    ) -> Result<tonic::Response<()>, tonic::Status> {
        let PutPartnerFees {
            id,
            fees: partner_fees,
        } = request.into_inner();
        let id = Uuid::from_str(&id).map_err(|_| AccountError::MalformedAccountToken)?;
        let fees = helpers::fees::PartnerFees(&self.pool);

        fees.set(id, partner_fees).await?;

        Ok(tonic::Response::new(()))
    }
}

enum AccountError {
    MalformedAccountToken,
    MalformedRoutingSourceToken,
    MalformedPaymentId,
    QueryFailed(String),
    PoolConnectionFailed,
    MissingAmount,
    CustomerNotFound,
    RetailerNotFound,
    TooManyRouteEntries,
    MissingRoutingData,
}

impl From<AccountError> for tonic::Status {
    fn from(value: AccountError) -> Self {
        match value {
            AccountError::MalformedAccountToken => {
                tonic::Status::invalid_argument("Malformed session token")
            }
            AccountError::MalformedRoutingSourceToken => {
                tonic::Status::invalid_argument("Malformed routing source token")
            }
            AccountError::MalformedPaymentId => {
                tonic::Status::invalid_argument("Malformed payment id")
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
            AccountError::TooManyRouteEntries => {
                tonic::Status::invalid_argument("There were more entries than profile indexes")
            }
            AccountError::MissingRoutingData => {
                tonic::Status::invalid_argument("Routing data was missing")
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
