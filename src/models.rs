use std::io::Write;

use diesel::{
    deserialize,
    pg::{Pg, PgValue},
    serialize, AsExpression, FromSqlRow, Insertable, Queryable,
};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::schema;

#[derive(Debug, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::KycLevel)]
pub enum KycLevel {
    Level0,
    Level1,
    Level2,
    Level3,
}

impl serialize::ToSql<crate::schema::sql_types::KycLevel, Pg> for KycLevel {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            KycLevel::Level0 => out.write_all(b"Level0")?,
            KycLevel::Level1 => out.write_all(b"Level1")?,
            KycLevel::Level2 => out.write_all(b"Level2")?,
            KycLevel::Level3 => out.write_all(b"Level3")?,
        }
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::schema::sql_types::KycLevel, Pg> for KycLevel {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Level0" => Ok(KycLevel::Level0),
            b"Level1" => Ok(KycLevel::Level1),
            b"Level2" => Ok(KycLevel::Level2),
            b"Level3" => Ok(KycLevel::Level3),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, AsExpression, FromSqlRow, Clone, Copy, Hash, PartialEq, Eq)]
#[diesel(sql_type = crate::schema::sql_types::Scope)]
pub enum ScopeKind {
    Public = 0,
    Customer = 1,
    Merchant = 2,
    Partner = 3,
    Admin = 4,
}

impl serialize::ToSql<crate::schema::sql_types::Scope, Pg> for ScopeKind {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            ScopeKind::Public => out.write_all(b"Public")?,
            ScopeKind::Customer => out.write_all(b"Customer")?,
            ScopeKind::Merchant => out.write_all(b"Merchant")?,
            ScopeKind::Partner => out.write_all(b"Partner")?,
            ScopeKind::Admin => out.write_all(b"Admin")?,
        }
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::schema::sql_types::Scope, Pg> for ScopeKind {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Public" => Ok(ScopeKind::Public),
            b"Customer" => Ok(ScopeKind::Customer),
            b"Merchant" => Ok(ScopeKind::Merchant),
            b"Partner" => Ok(ScopeKind::Partner),
            b"Admin" => Ok(ScopeKind::Admin),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = schema::accounts)]
pub struct Account<'s> {
    pub id: Uuid,
    pub email: &'s str,
}

#[derive(Queryable)]
pub struct Scope {
    pub account_id: Uuid,
    pub scope: ScopeKind,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = schema::email_login_intents)]
pub struct EmailLoginIntent<'s> {
    pub id: Uuid,
    pub account_id: Uuid,
    pub pass_key: &'s str,
    pub expires_at: OffsetDateTime,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = schema::new_pass_login_intents)]
pub struct NewPassLoginIntent<'s> {
    pub id: &'s str,
    pub account_id: Uuid,
    pub expires_at: OffsetDateTime,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = schema::password_login)]
pub struct PasswordLogin<'s> {
    pub account_id: Uuid,
    pub hash: &'s str,
    pub salt: &'s str,
    pub created_at: OffsetDateTime,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = schema::sessions)]
pub struct Session<'s> {
    pub token: &'s str,
    pub account_id: Uuid,
    pub password_login: bool,
    pub expires_at: OffsetDateTime,
}
