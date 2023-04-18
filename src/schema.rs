// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "approval"))]
    pub struct Approval;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "kyc_level"))]
    pub struct KycLevel;

    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "scope"))]
    pub struct Scope;
}

diesel::table! {
    accounts (id) {
        id -> Uuid,
        email -> Text,
        created_at -> Timestamptz,
        blocked -> Bool,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::KycLevel;
    use super::sql_types::Approval;

    customers (id) {
        id -> Uuid,
        first_name -> Text,
        last_name -> Text,
        kyc_level -> KycLevel,
        approved_at -> Nullable<Timestamptz>,
        approved -> Approval,
        residence_address -> Nullable<Text>,
        country_of_residence -> Nullable<Text>,
        account_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    email_login_intents (id) {
        id -> Uuid,
        account_id -> Uuid,
        pass_key -> Text,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    new_pass_login_intents (id) {
        id -> Text,
        account_id -> Uuid,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    password_login (account_id) {
        account_id -> Uuid,
        hash -> Text,
        salt -> Text,
        created_at -> Timestamptz,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Approval;

    retailers (id) {
        id -> Uuid,
        addr_line_1 -> Nullable<Text>,
        addr_line_2 -> Nullable<Text>,
        country -> Nullable<Text>,
        approved_at -> Nullable<Timestamptz>,
        approved -> Approval,
        account_id -> Nullable<Uuid>,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::Scope;

    scopes (account_id, scope) {
        account_id -> Uuid,
        scope -> Scope,
    }
}

diesel::table! {
    sessions (token) {
        token -> Text,
        account_id -> Uuid,
        password_login -> Bool,
        expires_at -> Timestamptz,
    }
}

diesel::table! {
    transactions (id) {
        id -> Uuid,
        retailer_id -> Nullable<Uuid>,
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
diesel::joinable!(email_login_intents -> accounts (account_id));
diesel::joinable!(new_pass_login_intents -> accounts (account_id));
diesel::joinable!(password_login -> accounts (account_id));
diesel::joinable!(retailers -> accounts (account_id));
diesel::joinable!(scopes -> accounts (account_id));
diesel::joinable!(sessions -> accounts (account_id));
diesel::joinable!(transactions -> retailers (retailer_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    customers,
    email_login_intents,
    new_pass_login_intents,
    password_login,
    retailers,
    scopes,
    sessions,
    transactions,
);
