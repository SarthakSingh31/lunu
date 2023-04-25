use std::io::Write;

use bigdecimal::BigDecimal;
use diesel::{
    deserialize,
    pg::{Pg, PgValue},
    serialize, AsChangeset, AsExpression, FromSqlRow, Insertable, Queryable,
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
    Retailer = 2,
    Partner = 3,
    Admin = 4,
}

impl serialize::ToSql<crate::schema::sql_types::Scope, Pg> for ScopeKind {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            ScopeKind::Public => out.write_all(b"Public")?,
            ScopeKind::Customer => out.write_all(b"Customer")?,
            ScopeKind::Retailer => out.write_all(b"Retailer")?,
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
            b"Retailer" => Ok(ScopeKind::Retailer),
            b"Partner" => Ok(ScopeKind::Partner),
            b"Admin" => Ok(ScopeKind::Admin),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, AsExpression, FromSqlRow, serde::Deserialize)]
#[diesel(sql_type = crate::schema::sql_types::Approval)]
pub enum Approval {
    OnHold = 0,
    Approved = 1,
    Rejected = 2,
}

impl serialize::ToSql<crate::schema::sql_types::Approval, Pg> for Approval {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            Approval::OnHold => out.write_all(b"OnHold")?,
            Approval::Approved => out.write_all(b"Approved")?,
            Approval::Rejected => out.write_all(b"Rejected")?,
        }
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::schema::sql_types::Approval, Pg> for Approval {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"OnHold" => Ok(Approval::OnHold),
            b"Approved" => Ok(Approval::Approved),
            b"Rejected" => Ok(Approval::Rejected),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, AsExpression, FromSqlRow, serde::Deserialize)]
#[diesel(sql_type = crate::schema::sql_types::LimitLevel)]
pub enum LimitLevel {
    KycLevel0 = 0,
    KycLevel1 = 1,
    KycLevel2 = 2,
    KycLevel3 = 3,
    Overall = 4,
}

impl serialize::ToSql<crate::schema::sql_types::LimitLevel, Pg> for LimitLevel {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            LimitLevel::KycLevel0 => out.write_all(b"KycLevel0")?,
            LimitLevel::KycLevel1 => out.write_all(b"KycLevel1")?,
            LimitLevel::KycLevel2 => out.write_all(b"KycLevel2")?,
            LimitLevel::KycLevel3 => out.write_all(b"KycLevel3")?,
            LimitLevel::Overall => out.write_all(b"Overall")?,
        }
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::schema::sql_types::LimitLevel, Pg> for LimitLevel {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"KycLevel0" => Ok(LimitLevel::KycLevel0),
            b"KycLevel1" => Ok(LimitLevel::KycLevel1),
            b"KycLevel2" => Ok(LimitLevel::KycLevel2),
            b"KycLevel3" => Ok(LimitLevel::KycLevel3),
            b"Overall" => Ok(LimitLevel::Overall),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, AsExpression, FromSqlRow, serde::Deserialize)]
#[diesel(sql_type = crate::schema::sql_types::LimitPeriod)]
pub enum LimitPeriod {
    Daily = 0,
    Weekly = 1,
    Monthly = 2,
}

impl serialize::ToSql<crate::schema::sql_types::LimitPeriod, Pg> for LimitPeriod {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            LimitPeriod::Daily => out.write_all(b"Daily")?,
            LimitPeriod::Weekly => out.write_all(b"Weekly")?,
            LimitPeriod::Monthly => out.write_all(b"Monthly")?,
        }
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::schema::sql_types::LimitPeriod, Pg> for LimitPeriod {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"Daily" => Ok(LimitPeriod::Daily),
            b"Weekly" => Ok(LimitPeriod::Weekly),
            b"Monthly" => Ok(LimitPeriod::Monthly),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

#[derive(Debug, AsExpression, FromSqlRow)]
#[diesel(sql_type = crate::schema::sql_types::ProfileIndex)]
pub enum ProfileIndex {
    Zero,
    One,
    Two,
}

impl serialize::ToSql<crate::schema::sql_types::ProfileIndex, Pg> for ProfileIndex {
    fn to_sql<'b>(&'b self, out: &mut serialize::Output<'b, '_, Pg>) -> serialize::Result {
        match *self {
            ProfileIndex::Zero => out.write_all(b"0")?,
            ProfileIndex::One => out.write_all(b"1")?,
            ProfileIndex::Two => out.write_all(b"2")?,
        }
        Ok(serialize::IsNull::No)
    }
}

impl deserialize::FromSql<crate::schema::sql_types::ProfileIndex, Pg> for ProfileIndex {
    fn from_sql(bytes: PgValue) -> deserialize::Result<Self> {
        match bytes.as_bytes() {
            b"0" => Ok(ProfileIndex::Zero),
            b"1" => Ok(ProfileIndex::One),
            b"2" => Ok(ProfileIndex::Two),
            _ => Err("Unrecognized enum variant".into()),
        }
    }
}

impl TryFrom<usize> for ProfileIndex {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ProfileIndex::Zero),
            1 => Ok(ProfileIndex::One),
            2 => Ok(ProfileIndex::Two),
            _ => Err(())
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

#[derive(Queryable, Insertable)]
#[diesel(table_name = schema::customers)]
pub struct Customer {
    pub id: Uuid,
    pub first_name: String,
    pub last_name: String,
    pub account_id: Uuid,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = schema::retailers)]
pub struct Retailer {
    pub id: Uuid,
    pub account_id: Uuid,
}

#[derive(Queryable, Insertable, AsChangeset)]
#[diesel(table_name = schema::customer_limits)]
pub struct CustomerLimit<'cl> {
    pub period: LimitPeriod,
    pub level: LimitLevel,
    pub amount: BigDecimal,
    pub currency: &'cl str,
    pub customer_id: Uuid,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = schema::retailer_limits)]
pub struct RetailerLimit<'rl> {
    pub period: LimitPeriod,
    pub level: LimitLevel,
    pub amount: BigDecimal,
    pub currency: &'rl str,
    pub retailer_id: Uuid,
}

#[derive(Queryable, Insertable)]
#[diesel(table_name = schema::global_limits)]
pub struct GlobalLimit<'rl> {
    pub period: LimitPeriod,
    pub level: LimitLevel,
    pub amount: BigDecimal,
    pub currency: &'rl str,
}
