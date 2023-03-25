// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "kyc_level"))]
    pub struct KycLevel;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::KycLevel;

    accounts (id) {
        id -> Uuid,
        created_at -> Timestamptz,
        approved_at -> Nullable<Timestamptz>,
        approved -> Bool,
        blocked -> Bool,
        kyc_level -> KycLevel,
        addr_line_1 -> Text,
        addr_line_2 -> Text,
        country -> Text,
    }
}

diesel::table! {
    customers (id) {
        id -> Uuid,
        first_name -> Text,
        last_name -> Text,
        email -> Text,
        total_risk -> Float4,
        personal_limit -> Nullable<Money>,
        personal_kyc_limit -> Nullable<Money>,
        account_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    merchants (id) {
        id -> Uuid,
        first_name -> Text,
        last_name -> Text,
        email -> Text,
        account_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    transactions (id) {
        id -> Uuid,
        merchant_id -> Nullable<Uuid>,
        retailer_transaction_id -> Nullable<Text>,
        retailer_customer_id -> Nullable<Text>,
        source_account_wallet -> Uuid,
        dest_account_wallet -> Uuid,
        kind -> Text,
        timestamp -> Timestamptz,
        payment_method -> Text,
        crypto_currency_type -> Text,
        crypto_network -> Text,
        crypto_amount -> Money,
        fiat_type -> Text,
        fiat_amount -> Money,
        exchange_rate -> Money,
        dest_crypto_address -> Int4,
        transcation_hash -> Int4,
        payment_gateway_fee -> Money,
        exchange_spread_fee -> Money,
        partner_fee -> Money,
        status -> Int4,
    }
}

diesel::joinable!(customers -> accounts (account_id));
diesel::joinable!(merchants -> accounts (account_id));
diesel::joinable!(transactions -> merchants (merchant_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    customers,
    merchants,
    transactions,
);
