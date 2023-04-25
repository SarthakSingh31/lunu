use std::str::FromStr;

use bigdecimal::BigDecimal;
use lunu::{
    account::{Routing, RoutingEntry},
    diesel::{insert_into, ExpressionMethods, JoinOnDsl, NullableExpressionMethods, QueryDsl},
    diesel_async::{pooled_connection::bb8::Pool, AsyncPgConnection, RunQueryDsl},
    models, schema,
};
use uuid::Uuid;

use crate::AccountError;

#[tonic::async_trait]
pub(crate) trait RoutingTable {
    type Id;

    async fn get(pool: &Pool<AsyncPgConnection>, id: Self::Id) -> Result<Routing, AccountError>;

    async fn set(
        pool: &Pool<AsyncPgConnection>,
        id: Self::Id,
        values: Routing,
    ) -> Result<(), AccountError>;
}

pub struct CustomerRouting;

#[tonic::async_trait]
impl RoutingTable for CustomerRouting {
    type Id = Uuid;

    async fn get(pool: &Pool<AsyncPgConnection>, id: Self::Id) -> Result<Routing, AccountError> {
        let conn = &mut pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::customer_payment_gateway_routing::dsl as cpgr_dsl;
        use schema::payment_gateways::dsl as pg_dsl;

        let payment_gateways = cpgr_dsl::customer_payment_gateway_routing
            .inner_join(pg_dsl::payment_gateways.on(pg_dsl::id.eq(cpgr_dsl::selected)))
            .filter(cpgr_dsl::customer_id.eq(id))
            .select((
                pg_dsl::id,
                pg_dsl::name,
                cpgr_dsl::currency,
                cpgr_dsl::amount,
            ))
            .load::<(Uuid, String, String, BigDecimal)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .into_iter()
            .map(|(id, name, currency, amount)| RoutingEntry {
                source: Some((id, name).into()),
                amount: Some((currency, amount).into()),
            })
            .collect();

        use schema::custody_providers::dsl as cp_dsl;
        use schema::customer_custody_provider_routing::dsl as ccpr_dsl;

        let custody_providers = ccpr_dsl::customer_custody_provider_routing
            .inner_join(cp_dsl::custody_providers.on(cp_dsl::id.eq(ccpr_dsl::selected)))
            .filter(ccpr_dsl::customer_id.eq(id))
            .select((
                cp_dsl::id,
                cp_dsl::name,
                ccpr_dsl::currency,
                ccpr_dsl::amount,
            ))
            .load::<(Uuid, String, String, BigDecimal)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .into_iter()
            .map(|(id, name, currency, amount)| RoutingEntry {
                source: Some((id, name).into()),
                amount: Some((currency, amount).into()),
            })
            .collect();

        use schema::customer_exchange_provider_routing::dsl as cepr_dsl;
        use schema::exchange_providers::dsl as ep_dsl;

        let exchange_providers = cepr_dsl::customer_exchange_provider_routing
            .inner_join(ep_dsl::exchange_providers.on(ep_dsl::id.eq(cepr_dsl::selected)))
            .filter(cepr_dsl::customer_id.eq(id))
            .select((
                ep_dsl::id,
                ep_dsl::name,
                cepr_dsl::currency,
                cepr_dsl::amount,
            ))
            .load::<(Uuid, String, String, BigDecimal)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .into_iter()
            .map(|(id, name, currency, amount)| RoutingEntry {
                source: Some((id, name).into()),
                amount: Some((currency, amount).into()),
            })
            .collect();

        Ok(Routing {
            payment_gateways,
            custody_providers,
            exchange_providers,
        })
    }

    async fn set(
        pool: &Pool<AsyncPgConnection>,
        id: Self::Id,
        Routing {
            payment_gateways,
            custody_providers,
            exchange_providers,
        }: Routing,
    ) -> Result<(), AccountError> {
        let conn = &mut pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::customer_payment_gateway_routing::dsl as cpgr_dsl;

        let mut parsed_payment_gateways = Vec::with_capacity(payment_gateways.len());

        for (idx, entry) in payment_gateways.into_iter().enumerate() {
            let Some(source) = entry.source else {
                continue;
            };
            let Some(amount) = entry.amount else {
                continue;
            };
            let selected_id = Uuid::from_str(&source.id)
                .map_err(|_| AccountError::MalformedRoutingSourceToken)?;
            let (currency, amount): (String, BigDecimal) = amount.into();

            parsed_payment_gateways.push((
                cpgr_dsl::idx.eq(models::ProfileIndex::try_from(idx)
                    .map_err(|_| AccountError::TooManyRouteEntries)?),
                cpgr_dsl::customer_id.eq(id),
                cpgr_dsl::selected.eq(selected_id),
                cpgr_dsl::amount.eq(amount),
                cpgr_dsl::currency.eq(currency),
            ));
        }

        insert_into(cpgr_dsl::customer_payment_gateway_routing)
            .values(&parsed_payment_gateways)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        use schema::customer_custody_provider_routing::dsl as ccpr_dsl;

        let mut parsed_custody_providers = Vec::with_capacity(custody_providers.len());

        for (idx, entry) in custody_providers.into_iter().enumerate() {
            let Some(source) = entry.source else {
                    continue;
                };
            let Some(amount) = entry.amount else {
                    continue;
                };
            let selected_id = Uuid::from_str(&source.id)
                .map_err(|_| AccountError::MalformedRoutingSourceToken)?;
            let (currency, amount): (String, BigDecimal) = amount.into();

            parsed_custody_providers.push((
                ccpr_dsl::idx.eq(models::ProfileIndex::try_from(idx)
                    .map_err(|_| AccountError::TooManyRouteEntries)?),
                ccpr_dsl::customer_id.eq(id),
                ccpr_dsl::selected.eq(selected_id),
                ccpr_dsl::amount.eq(amount),
                ccpr_dsl::currency.eq(currency),
            ));
        }

        insert_into(ccpr_dsl::customer_custody_provider_routing)
            .values(&parsed_custody_providers)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        use schema::customer_exchange_provider_routing::dsl as cepr_dsl;

        let mut parsed_exchange_providers = Vec::with_capacity(exchange_providers.len());

        for (idx, entry) in exchange_providers.into_iter().enumerate() {
            let Some(source) = entry.source else {
                        continue;
                    };
            let Some(amount) = entry.amount else {
                        continue;
                    };
            let selected_id = Uuid::from_str(&source.id)
                .map_err(|_| AccountError::MalformedRoutingSourceToken)?;
            let (currency, amount): (String, BigDecimal) = amount.into();

            parsed_exchange_providers.push((
                cepr_dsl::idx.eq(models::ProfileIndex::try_from(idx)
                    .map_err(|_| AccountError::TooManyRouteEntries)?),
                cepr_dsl::customer_id.eq(id),
                cepr_dsl::selected.eq(selected_id),
                cepr_dsl::amount.eq(amount),
                cepr_dsl::currency.eq(currency),
            ));
        }

        insert_into(cepr_dsl::customer_exchange_provider_routing)
            .values(&parsed_exchange_providers)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}

pub struct RetailerRouting;

#[tonic::async_trait]
impl RoutingTable for RetailerRouting {
    type Id = Uuid;

    async fn get(pool: &Pool<AsyncPgConnection>, id: Self::Id) -> Result<Routing, AccountError> {
        let conn = &mut pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::payment_gateways::dsl as pg_dsl;
        use schema::retailer_payment_gateway_routing::dsl as rpgr_dsl;

        let payment_gateways = rpgr_dsl::retailer_payment_gateway_routing
            .inner_join(pg_dsl::payment_gateways.on(pg_dsl::id.eq(rpgr_dsl::selected)))
            .filter(rpgr_dsl::retailer_id.eq(id))
            .select((
                pg_dsl::id,
                pg_dsl::name,
                rpgr_dsl::currency,
                rpgr_dsl::amount,
            ))
            .load::<(Uuid, String, String, BigDecimal)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .into_iter()
            .map(|(id, name, currency, amount)| RoutingEntry {
                source: Some((id, name).into()),
                amount: Some((currency, amount).into()),
            })
            .collect();

        use schema::custody_providers::dsl as cp_dsl;
        use schema::retailer_custody_provider_routing::dsl as rcpr_dsl;

        let custody_providers = rcpr_dsl::retailer_custody_provider_routing
            .inner_join(cp_dsl::custody_providers.on(cp_dsl::id.eq(rcpr_dsl::selected)))
            .filter(rcpr_dsl::retailer_id.eq(id))
            .select((
                cp_dsl::id,
                cp_dsl::name,
                rcpr_dsl::currency,
                rcpr_dsl::amount,
            ))
            .load::<(Uuid, String, String, BigDecimal)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .into_iter()
            .map(|(id, name, currency, amount)| RoutingEntry {
                source: Some((id, name).into()),
                amount: Some((currency, amount).into()),
            })
            .collect();

        use schema::exchange_providers::dsl as ep_dsl;
        use schema::retailer_exchange_provider_routing::dsl as repr_dsl;

        let exchange_providers = repr_dsl::retailer_exchange_provider_routing
            .inner_join(ep_dsl::exchange_providers.on(ep_dsl::id.eq(repr_dsl::selected)))
            .filter(repr_dsl::retailer_id.eq(id))
            .select((
                ep_dsl::id,
                ep_dsl::name,
                repr_dsl::currency,
                repr_dsl::amount,
            ))
            .load::<(Uuid, String, String, BigDecimal)>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .into_iter()
            .map(|(id, name, currency, amount)| RoutingEntry {
                source: Some((id, name).into()),
                amount: Some((currency, amount).into()),
            })
            .collect();

        Ok(Routing {
            payment_gateways,
            custody_providers,
            exchange_providers,
        })
    }

    async fn set(
        pool: &Pool<AsyncPgConnection>,
        id: Self::Id,
        Routing {
            payment_gateways,
            custody_providers,
            exchange_providers,
        }: Routing,
    ) -> Result<(), AccountError> {
        let conn = &mut pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::retailer_payment_gateway_routing::dsl as rpgr_dsl;

        let mut parsed_payment_gateways = Vec::with_capacity(payment_gateways.len());

        for (idx, entry) in payment_gateways.into_iter().enumerate() {
            let Some(source) = entry.source else {
                continue;
            };
            let Some(amount) = entry.amount else {
                continue;
            };
            let selected_id = Uuid::from_str(&source.id)
                .map_err(|_| AccountError::MalformedRoutingSourceToken)?;
            let (currency, amount): (String, BigDecimal) = amount.into();

            parsed_payment_gateways.push((
                rpgr_dsl::idx.eq(models::ProfileIndex::try_from(idx)
                    .map_err(|_| AccountError::TooManyRouteEntries)?),
                rpgr_dsl::retailer_id.eq(id),
                rpgr_dsl::selected.eq(selected_id),
                rpgr_dsl::amount.eq(amount),
                rpgr_dsl::currency.eq(currency),
            ));
        }

        insert_into(rpgr_dsl::retailer_payment_gateway_routing)
            .values(&parsed_payment_gateways)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        use schema::retailer_custody_provider_routing::dsl as rcpr_dsl;

        let mut parsed_custody_providers = Vec::with_capacity(custody_providers.len());

        for (idx, entry) in custody_providers.into_iter().enumerate() {
            let Some(source) = entry.source else {
                    continue;
                };
            let Some(amount) = entry.amount else {
                    continue;
                };
            let selected_id = Uuid::from_str(&source.id)
                .map_err(|_| AccountError::MalformedRoutingSourceToken)?;
            let (currency, amount): (String, BigDecimal) = amount.into();

            parsed_custody_providers.push((
                rcpr_dsl::idx.eq(models::ProfileIndex::try_from(idx)
                    .map_err(|_| AccountError::TooManyRouteEntries)?),
                rcpr_dsl::retailer_id.eq(id),
                rcpr_dsl::selected.eq(selected_id),
                rcpr_dsl::amount.eq(amount),
                rcpr_dsl::currency.eq(currency),
            ));
        }

        insert_into(rcpr_dsl::retailer_custody_provider_routing)
            .values(&parsed_custody_providers)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        use schema::retailer_exchange_provider_routing::dsl as repr_dsl;

        let mut parsed_exchange_providers = Vec::with_capacity(exchange_providers.len());

        for (idx, entry) in exchange_providers.into_iter().enumerate() {
            let Some(source) = entry.source else {
                        continue;
                    };
            let Some(amount) = entry.amount else {
                        continue;
                    };
            let selected_id = Uuid::from_str(&source.id)
                .map_err(|_| AccountError::MalformedRoutingSourceToken)?;
            let (currency, amount): (String, BigDecimal) = amount.into();

            parsed_exchange_providers.push((
                repr_dsl::idx.eq(models::ProfileIndex::try_from(idx)
                    .map_err(|_| AccountError::TooManyRouteEntries)?),
                repr_dsl::retailer_id.eq(id),
                repr_dsl::selected.eq(selected_id),
                repr_dsl::amount.eq(amount),
                repr_dsl::currency.eq(currency),
            ));
        }

        insert_into(repr_dsl::retailer_exchange_provider_routing)
            .values(&parsed_exchange_providers)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}

pub struct GlobalRouting;

#[tonic::async_trait]
impl RoutingTable for GlobalRouting {
    type Id = ();

    async fn get(pool: &Pool<AsyncPgConnection>, _id: Self::Id) -> Result<Routing, AccountError> {
        let conn = &mut pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::global_payment_gateway_routing::dsl as gpgr_dsl;
        use schema::payment_gateways::dsl as pg_dsl;

        let payment_gateways = gpgr_dsl::global_payment_gateway_routing
            .left_join(pg_dsl::payment_gateways.on(gpgr_dsl::selected.eq(pg_dsl::id.nullable())))
            .select((
                pg_dsl::id.nullable(),
                pg_dsl::name.nullable(),
                gpgr_dsl::currency,
                gpgr_dsl::amount,
            ))
            .load::<(
                Option<Uuid>,
                Option<String>,
                Option<String>,
                Option<BigDecimal>,
            )>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .into_iter()
            .map(|(id, name, currency, amount)| RoutingEntry {
                source: id.zip(name).map(|(id, name)| (id, name).into()),
                amount: currency
                    .zip(amount)
                    .map(|(currency, amount)| (currency, amount).into()),
            })
            .collect();

        use schema::custody_providers::dsl as cp_dsl;
        use schema::global_custody_provider_routing::dsl as gcpr_dsl;

        let custody_providers = gcpr_dsl::global_custody_provider_routing
            .inner_join(cp_dsl::custody_providers.on(cp_dsl::id.nullable().eq(gcpr_dsl::selected)))
            .select((
                cp_dsl::id.nullable(),
                cp_dsl::name.nullable(),
                gcpr_dsl::currency,
                gcpr_dsl::amount,
            ))
            .load::<(
                Option<Uuid>,
                Option<String>,
                Option<String>,
                Option<BigDecimal>,
            )>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .into_iter()
            .map(|(id, name, currency, amount)| RoutingEntry {
                source: id.zip(name).map(|(id, name)| (id, name).into()),
                amount: currency
                    .zip(amount)
                    .map(|(currency, amount)| (currency, amount).into()),
            })
            .collect();

        use schema::exchange_providers::dsl as ep_dsl;
        use schema::global_exchange_provider_routing::dsl as gepr_dsl;

        let exchange_providers = gepr_dsl::global_exchange_provider_routing
            .inner_join(ep_dsl::exchange_providers.on(ep_dsl::id.nullable().eq(gepr_dsl::selected)))
            .select((
                ep_dsl::id.nullable(),
                ep_dsl::name.nullable(),
                gepr_dsl::currency,
                gepr_dsl::amount,
            ))
            .load::<(
                Option<Uuid>,
                Option<String>,
                Option<String>,
                Option<BigDecimal>,
            )>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?
            .into_iter()
            .map(|(id, name, currency, amount)| RoutingEntry {
                source: id.zip(name).map(|(id, name)| (id, name).into()),
                amount: currency
                    .zip(amount)
                    .map(|(currency, amount)| (currency, amount).into()),
            })
            .collect();

        Ok(Routing {
            payment_gateways,
            custody_providers,
            exchange_providers,
        })
    }

    async fn set(
        pool: &Pool<AsyncPgConnection>,
        _id: Self::Id,
        Routing {
            payment_gateways,
            custody_providers,
            exchange_providers,
        }: Routing,
    ) -> Result<(), AccountError> {
        let conn = &mut pool
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::global_payment_gateway_routing::dsl as gpgr_dsl;

        let mut parsed_payment_gateways = Vec::with_capacity(payment_gateways.len());

        for (idx, entry) in payment_gateways.into_iter().enumerate() {
            let Some(source) = entry.source else {
            continue;
        };
            let Some(amount) = entry.amount else {
            continue;
        };
            let selected_id = Uuid::from_str(&source.id)
                .map_err(|_| AccountError::MalformedRoutingSourceToken)?;
            let (currency, amount): (String, BigDecimal) = amount.into();

            parsed_payment_gateways.push((
                gpgr_dsl::idx.eq(models::ProfileIndex::try_from(idx)
                    .map_err(|_| AccountError::TooManyRouteEntries)?),
                gpgr_dsl::selected.eq(selected_id),
                gpgr_dsl::amount.eq(amount),
                gpgr_dsl::currency.eq(currency),
            ));
        }

        insert_into(gpgr_dsl::global_payment_gateway_routing)
            .values(&parsed_payment_gateways)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        use schema::global_custody_provider_routing::dsl as gcpr_dsl;

        let mut parsed_custody_providers = Vec::with_capacity(custody_providers.len());

        for (idx, entry) in custody_providers.into_iter().enumerate() {
            let Some(source) = entry.source else {
                continue;
            };
            let Some(amount) = entry.amount else {
                continue;
            };
            let selected_id = Uuid::from_str(&source.id)
                .map_err(|_| AccountError::MalformedRoutingSourceToken)?;
            let (currency, amount): (String, BigDecimal) = amount.into();

            parsed_custody_providers.push((
                gcpr_dsl::idx.eq(models::ProfileIndex::try_from(idx)
                    .map_err(|_| AccountError::TooManyRouteEntries)?),
                gcpr_dsl::selected.eq(selected_id),
                gcpr_dsl::amount.eq(amount),
                gcpr_dsl::currency.eq(currency),
            ));
        }

        insert_into(gcpr_dsl::global_custody_provider_routing)
            .values(&parsed_custody_providers)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        use schema::global_exchange_provider_routing::dsl as gepr_dsl;

        let mut parsed_exchange_providers = Vec::with_capacity(exchange_providers.len());

        for (idx, entry) in exchange_providers.into_iter().enumerate() {
            let Some(source) = entry.source else {
                    continue;
                };
            let Some(amount) = entry.amount else {
                    continue;
                };
            let selected_id = Uuid::from_str(&source.id)
                .map_err(|_| AccountError::MalformedRoutingSourceToken)?;
            let (currency, amount): (String, BigDecimal) = amount.into();

            parsed_exchange_providers.push((
                gepr_dsl::idx.eq(models::ProfileIndex::try_from(idx)
                    .map_err(|_| AccountError::TooManyRouteEntries)?),
                gepr_dsl::selected.eq(selected_id),
                gepr_dsl::amount.eq(amount),
                gepr_dsl::currency.eq(currency),
            ));
        }

        insert_into(gepr_dsl::global_exchange_provider_routing)
            .values(&parsed_exchange_providers)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}
