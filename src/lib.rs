#[cfg(feature = "db")]
pub mod models;
#[cfg(feature = "db")]
pub mod schema;

use std::collections::HashMap;

#[cfg(feature = "db")]
pub use diesel;
#[cfg(feature = "db")]
pub use diesel_async;
pub use dotenvy;

#[derive(Debug, Hash, PartialEq, Eq)]
pub enum Microservice {
    Auth,
    Account,
    Storage,
    Email,
}

lazy_static::lazy_static! {
    pub static ref MICROSERVICE_ADDRS: HashMap<Microservice, &'static str> = [
        (Microservice::Auth, "[::1]:50051"),
        (Microservice::Account, "[::1]:50052"),
        (Microservice::Storage, "[::1]:50053"),
        (Microservice::Email, "[::1]:50054"),
    ].into_iter().collect();
}

#[macro_export]
macro_rules! register_tonic_clients {
    ($(($name:ident, $client:ty, $varient:expr, $module:expr)),*$(,)?) => {
        $(register_tonic_clients!{static $name, $client})*

        async fn init_clients() {
            $(register_tonic_clients!{async $name, $client, $varient, $module})*
        }
    };
    (static $name:ident, $client:ty) => {
        static $name: tokio::sync::OnceCell<$client> = tokio::sync::OnceCell::const_new();
    };
    (async $name:ident, $client:ty, $varient:expr, $module:expr) => {
        let client = loop {
            match <$client>::connect(format!(
                "http://{}",
                lunu::MICROSERVICE_ADDRS[&$varient],
            ))
            .await
            {
                Ok(client) => break client,
                Err(_) => tokio::time::sleep(std::time::Duration::from_secs(1)).await,
            }
        };

        $name.set(client).expect(&format!("Failed to init {} client.", $module));

        println!("Connected to {} microservice!", $module);
    };
}

#[cfg(feature = "auth")]
pub mod auth {
    tonic::include_proto!("auth");
}

#[cfg(feature = "account")]
pub mod account {
    use bigdecimal::{num_bigint::BigInt, BigDecimal};

    tonic::include_proto!("account");

    #[cfg(feature = "db")]
    impl From<Approval> for super::models::Approval {
        fn from(val: Approval) -> super::models::Approval {
            match val {
                Approval::Approved => super::models::Approval::Approved,
                Approval::Rejected => super::models::Approval::Rejected,
                Approval::OnHold => super::models::Approval::OnHold,
            }
        }
    }

    #[cfg(feature = "db")]
    impl From<super::models::Approval> for Approval {
        fn from(val: super::models::Approval) -> Approval {
            match val {
                super::models::Approval::Approved => Approval::Approved,
                super::models::Approval::Rejected => Approval::Rejected,
                super::models::Approval::OnHold => Approval::OnHold,
            }
        }
    }

    #[cfg(feature = "db")]
    impl From<LimitLevel> for super::models::LimitLevel {
        fn from(val: LimitLevel) -> super::models::LimitLevel {
            match val {
                LimitLevel::KycLevel0 => super::models::LimitLevel::KycLevel0,
                LimitLevel::KycLevel1 => super::models::LimitLevel::KycLevel1,
                LimitLevel::KycLevel2 => super::models::LimitLevel::KycLevel2,
                LimitLevel::KycLevel3 => super::models::LimitLevel::KycLevel3,
                LimitLevel::Overall => super::models::LimitLevel::Overall,
            }
        }
    }

    #[cfg(feature = "db")]
    impl From<LimitPeriod> for super::models::LimitPeriod {
        fn from(val: LimitPeriod) -> super::models::LimitPeriod {
            match val {
                LimitPeriod::Daily => super::models::LimitPeriod::Daily,
                LimitPeriod::Weekly => super::models::LimitPeriod::Weekly,
                LimitPeriod::Monthly => super::models::LimitPeriod::Monthly,
            }
        }
    }

    #[cfg(feature = "db")]
    impl From<super::models::LimitLevel> for LimitLevel {
        fn from(val: super::models::LimitLevel) -> LimitLevel {
            match val {
                super::models::LimitLevel::KycLevel0 => LimitLevel::KycLevel0,
                super::models::LimitLevel::KycLevel1 => LimitLevel::KycLevel1,
                super::models::LimitLevel::KycLevel2 => LimitLevel::KycLevel2,
                super::models::LimitLevel::KycLevel3 => LimitLevel::KycLevel3,
                super::models::LimitLevel::Overall => LimitLevel::Overall,
            }
        }
    }

    #[cfg(feature = "db")]
    impl From<super::models::LimitPeriod> for LimitPeriod {
        fn from(val: super::models::LimitPeriod) -> LimitPeriod {
            match val {
                super::models::LimitPeriod::Daily => LimitPeriod::Daily,
                super::models::LimitPeriod::Weekly => LimitPeriod::Weekly,
                super::models::LimitPeriod::Monthly => LimitPeriod::Monthly,
            }
        }
    }

    impl From<u8> for LimitPeriod {
        fn from(value: u8) -> Self {
            match value {
                x if x == LimitPeriod::Daily as u8 => LimitPeriod::Daily,
                x if x == LimitPeriod::Weekly as u8 => LimitPeriod::Weekly,
                x if x == LimitPeriod::Monthly as u8 => LimitPeriod::Monthly,
                _ => panic!("LimitPeriod got an invalid value in converting to limits"),
            }
        }
    }

    impl From<u8> for LimitLevel {
        fn from(value: u8) -> Self {
            match value {
                x if x == LimitLevel::KycLevel0 as u8 => LimitLevel::KycLevel0,
                x if x == LimitLevel::KycLevel1 as u8 => LimitLevel::KycLevel1,
                x if x == LimitLevel::KycLevel2 as u8 => LimitLevel::KycLevel2,
                x if x == LimitLevel::KycLevel3 as u8 => LimitLevel::KycLevel3,
                x if x == LimitLevel::Overall as u8 => LimitLevel::Overall,
                _ => panic!("LimitLevel got an invalid value in converting to limits"),
            }
        }
    }

    #[derive(serde::Serialize)]
    #[serde(transparent)]
    pub struct Limits(pub super::HashMap<(LimitPeriod, LimitLevel), Money>);

    impl From<InnerLimits> for Limits {
        fn from(val: InnerLimits) -> Self {
            Limits(
                val.limit_map
                    .into_iter()
                    .map(|(key, value)| {
                        let [_, _, period, level] = key.to_le_bytes();
                        ((period.into(), level.into()), value)
                    })
                    .collect(),
            )
        }
    }

    impl From<Limits> for InnerLimits {
        fn from(val: Limits) -> Self {
            InnerLimits {
                limit_map: val
                    .0
                    .into_iter()
                    .map(|(key, value)| {
                        (u32::from_le_bytes([0, 0, key.0 as u8, key.1 as u8]), value)
                    })
                    .collect(),
            }
        }
    }

    impl From<(String, BigDecimal)> for Money {
        fn from((currency_code, amount): (String, BigDecimal)) -> Self {
            let (digits, exponent) = amount.into_bigint_and_exponent();
            Money {
                currency_code,
                digits: digits.to_signed_bytes_le(),
                exponent,
            }
        }
    }

    impl Into<(String, BigDecimal)> for Money {
        fn into(self) -> (String, BigDecimal) {
            (
                self.currency_code,
                BigDecimal::new(BigInt::from_signed_bytes_le(&self.digits), self.exponent),
            )
        }
    }
}

#[cfg(feature = "storage")]
pub mod storage {
    tonic::include_proto!("storage");
}

#[cfg(feature = "email")]
pub mod email {
    tonic::include_proto!("email");
}
