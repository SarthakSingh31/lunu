use std::str::FromStr;

use bigdecimal::BigDecimal;
use lunu::{
    account::{self, PaymentMethod},
    diesel::{
        insert_into, BoolExpressionMethods, ExpressionMethods, JoinOnDsl,
        NullableExpressionMethods, QueryDsl,
    },
    diesel_async::{pooled_connection::bb8::Pool, AsyncPgConnection, RunQueryDsl},
    schema,
};
use uuid::Uuid;

use crate::AccountError;

pub struct RetailerFees<'r>(pub &'r Pool<AsyncPgConnection>);

impl<'r> RetailerFees<'r> {
    pub(crate) async fn get(&self, id: Uuid) -> Result<account::RetailerFees, AccountError> {
        let conn = &mut self
            .0
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::payment_methods::dsl as pm_dsl;
        use schema::retailer_fees::dsl as rf_dsl;

        let fees = pm_dsl::payment_methods
            .left_join(rf_dsl::retailer_fees.on(rf_dsl::payment_method_id.eq(pm_dsl::id)))
            .filter(rf_dsl::retailer_id.eq(id))
            .select((
                pm_dsl::id,
                pm_dsl::name,
                rf_dsl::retailer_fee.nullable(),
                rf_dsl::consumer_fee.nullable(),
                rf_dsl::exchange_spread.nullable(),
                rf_dsl::exchange_spread_stable_coin.nullable(),
                rf_dsl::min_transaction_fee.nullable(),
                rf_dsl::base_additional_fixed_fee_amount.nullable(),
                rf_dsl::base_additional_fixed_fee_currency.nullable(),
            ))
            .load::<(
                Uuid,
                String,
                Option<BigDecimal>,
                Option<BigDecimal>,
                Option<BigDecimal>,
                Option<BigDecimal>,
                Option<BigDecimal>,
                Option<BigDecimal>,
                Option<String>,
            )>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        use schema::partner_fees::dsl as pf_dsl;
        use schema::retailer_partners::dsl as rp_dsl;

        let referral_partner_fees = pm_dsl::payment_methods
            .left_join(rf_dsl::retailer_fees.on(rf_dsl::payment_method_id.eq(pm_dsl::id)))
            .filter(rp_dsl::retailer_id.eq(id))
            .left_join(rp_dsl::retailer_partners.on(rp_dsl::retailer_id.eq(rf_dsl::retailer_id)))
            .left_join(
                pf_dsl::partner_fees.on(pf_dsl::partner_id
                    .eq(rp_dsl::partner_id)
                    .and(pf_dsl::payment_method_id.eq(rf_dsl::payment_method_id))),
            )
            .select((
                pm_dsl::id,
                rp_dsl::partner_id.nullable(),
                pf_dsl::referral_partner_fee.nullable(),
                pf_dsl::additional_fixed_fee_amount.nullable(),
                pf_dsl::additional_fixed_fee_currency.nullable(),
            ))
            .load::<(
                Uuid,
                Option<Uuid>,
                Option<BigDecimal>,
                Option<BigDecimal>,
                Option<String>,
            )>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(account::RetailerFees {
            fees: fees
                .into_iter()
                .map(|fee| account::RetailerFeeEntry {
                    payment_method: Some(PaymentMethod {
                        id: fee.0.to_string(),
                        name: fee.1,
                    }),
                    retailer_fee: fee.2.map(|f| f.into()),
                    consumer_fee: fee.3.map(|f| f.into()),
                    exchange_spread: fee.4.map(|f| f.into()),
                    exchange_spread_stable_coins: fee.5.map(|f| f.into()),
                    min_transaction_fee: fee.6.map(|f| f.into()),
                    referral_partner_fees: referral_partner_fees
                        .iter()
                        .filter(|f| f.0 == fee.0)
                        .filter_map(|fee| {
                            Some(account::PartnerRetailerFeeEntry {
                                partner_id: fee.1?.to_string(),
                                partner_fee: fee.2.as_ref().map(|f| f.clone().into()),
                                additional_fixed_fee: fee
                                    .4
                                    .as_ref()
                                    .zip(fee.3.as_ref())
                                    .map(|(c, a)| (c.clone(), a.clone()).into()),
                            })
                        })
                        .collect(),
                    additional_fixed_fee: fee.8.zip(fee.7).map(|(c, a)| (c, a).into()),
                })
                .collect(),
        })
    }

    pub(crate) async fn set(
        &self,
        id: Uuid,
        fees: Vec<account::PutRetailerFeeEntry>,
    ) -> Result<(), AccountError> {
        let conn = &mut self
            .0
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::retailer_fees::dsl as rf_dsl;

        let mut parsed_fees = Vec::with_capacity(fees.len());
        for fee in fees {
            let payment_method_id = Uuid::from_str(&fee.payment_method_id)
                .map_err(|_| AccountError::MalformedPaymentId)?;

            let Some(retailer_fee) = fee.retailer_fee else {
                continue;
            };
            let retailer_fee: BigDecimal = retailer_fee.into();

            let Some(consumer_fee) = fee.consumer_fee else {
                continue;
            };
            let consumer_fee: BigDecimal = consumer_fee.into();

            let Some(exchange_spread) = fee.exchange_spread else {
                continue;
            };
            let exchange_spread: BigDecimal = exchange_spread.into();

            let Some(exchange_spread_stable_coin) = fee.exchange_spread_stable_coins else {
                continue;
            };
            let exchange_spread_stable_coin: BigDecimal = exchange_spread_stable_coin.into();

            let Some(min_transaction_fee) = fee.min_transaction_fee else {
                continue;
            };
            let min_transaction_fee: BigDecimal = min_transaction_fee.into();

            let Some(additional_fixed_fee) = fee.additional_fixed_fee else {
                continue;
            };
            let (currency, amount): (String, BigDecimal) = additional_fixed_fee.into();

            parsed_fees.push((
                rf_dsl::payment_method_id.eq(payment_method_id),
                rf_dsl::retailer_id.eq(id),
                rf_dsl::retailer_fee.eq(retailer_fee),
                rf_dsl::consumer_fee.eq(consumer_fee),
                rf_dsl::exchange_spread.eq(exchange_spread),
                rf_dsl::exchange_spread_stable_coin.eq(exchange_spread_stable_coin),
                rf_dsl::min_transaction_fee.eq(min_transaction_fee),
                rf_dsl::base_additional_fixed_fee_amount.eq(amount),
                rf_dsl::base_additional_fixed_fee_currency.eq(currency),
            ));
        }

        insert_into(rf_dsl::retailer_fees)
            .values(parsed_fees)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}

pub struct PartnerFees<'r>(pub &'r Pool<AsyncPgConnection>);

impl<'r> PartnerFees<'r> {
    pub(crate) async fn get(&self, id: Uuid) -> Result<account::PartnerFees, AccountError> {
        let conn = &mut self
            .0
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::partner_fees::dsl as pf_dsl;
        use schema::payment_methods::dsl as pm_dsl;

        let fees = pm_dsl::payment_methods
            .left_join(pf_dsl::partner_fees.on(pf_dsl::payment_method_id.eq(pm_dsl::id)))
            .filter(pf_dsl::partner_id.eq(id))
            .select((
                pm_dsl::id,
                pm_dsl::name,
                pf_dsl::referral_partner_fee.nullable(),
                pf_dsl::additional_fixed_fee_amount.nullable(),
                pf_dsl::additional_fixed_fee_currency.nullable(),
            ))
            .load::<(
                Uuid,
                String,
                Option<BigDecimal>,
                Option<BigDecimal>,
                Option<String>,
            )>(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(account::PartnerFees {
            fees: fees
                .into_iter()
                .map(|fee| account::PartnerFeeEntry {
                    payment_method: Some(PaymentMethod {
                        id: fee.0.to_string(),
                        name: fee.1,
                    }),
                    partner_fee: fee.2.map(|f| f.into()),
                    additional_fixed_fee: fee.4.zip(fee.3).map(|(c, a)| (c, a).into()),
                })
                .collect(),
        })
    }

    pub(crate) async fn set(
        &self,
        id: Uuid,
        fees: Vec<account::PutPartnerFeeEntry>,
    ) -> Result<(), AccountError> {
        let conn = &mut self
            .0
            .get()
            .await
            .map_err(|_| AccountError::PoolConnectionFailed)?;

        use schema::partner_fees::dsl as pf_dsl;

        let mut parsed_fees = Vec::with_capacity(fees.len());
        for fee in fees {
            let payment_method_id = Uuid::from_str(&fee.payment_method_id)
                .map_err(|_| AccountError::MalformedPaymentId)?;

            let Some(referral_partner_fee) = fee.partner_fee else {
                continue;
            };
            let Some(additional_fixed_fee) = fee.additional_fixed_fee else {
                continue;
            };
            let referral_partner_fee: BigDecimal = referral_partner_fee.into();
            let (currency, amount): (String, BigDecimal) = additional_fixed_fee.into();

            parsed_fees.push((
                pf_dsl::payment_method_id.eq(payment_method_id),
                pf_dsl::partner_id.eq(id),
                pf_dsl::referral_partner_fee.eq(referral_partner_fee),
                pf_dsl::additional_fixed_fee_amount.eq(amount),
                pf_dsl::additional_fixed_fee_currency.eq(currency),
            ));
        }

        insert_into(pf_dsl::partner_fees)
            .values(parsed_fees)
            .execute(conn)
            .await
            .map_err(|e| AccountError::QueryFailed(e.to_string()))?;

        Ok(())
    }
}
