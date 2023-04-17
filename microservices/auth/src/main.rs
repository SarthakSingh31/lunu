use std::{cell::RefCell, env, ops::DerefMut, str::FromStr};

use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use lunu::{
    auth::{
        auth_server::AuthServer, Account, AccountEmail, EmailLoginIntent, EmailLoginParams, Empty,
        NewPassLoginParams, OptionalAccount, PasswordParams, SessionToken,
    },
    diesel::{
        delete, insert_into, pg::Pg, ExpressionMethods, JoinOnDsl, OptionalExtension, QueryDsl,
    },
    diesel_async::{
        pooled_connection::{bb8::Pool, AsyncDieselConnectionManager},
        AsyncConnection, AsyncPgConnection, RunQueryDsl,
    },
    dotenvy::dotenv,
    email::Email,
    models, register_tonic_clients, schema, Microservice, MICROSERVICE_ADDRS,
};
use rand::{
    distributions::Alphanumeric,
    rngs::{OsRng, ThreadRng},
    thread_rng, Rng,
};
use time::{Duration, OffsetDateTime};
use tonic::transport::{Channel, Server};
use uuid::Uuid;

thread_local! {
    static RNG: RefCell<ThreadRng> = RefCell::new(thread_rng());
}

register_tonic_clients! {
    (MAIL_CLIENT, lunu::email::mail_client::MailClient<Channel>, lunu::Microservice::Email, "email"),
}

struct Auth {
    pool: Pool<AsyncPgConnection>,
}

impl Auth {
    // Setting the session intent duration to 6 minutes (5 minutes + 1 minute grace)
    const SESSION_INTENT_DURATION: Duration = Duration::minutes(6);
    // Setting the session intent code length
    const SESSION_INTENT_CODE_LEN: usize = 6;
    // Setting the session duration to 1 week
    const SESSION_DURATION: Duration = Duration::WEEK;
    // Setting the session token length
    const SESSION_TOKEN_LEN: usize = 128;
    // Setting the new password login token length
    const NEW_PASS_LOGIN_TOKEN_LEN: usize = 64;

    async fn get_account(
        &self,
        conn: &mut (impl AsyncConnection<Backend = Pg> + Send),
        email: &str,
    ) -> Result<Option<Uuid>, tonic::Status> {
        use schema::accounts::dsl as a_dsl;

        if let Some(account_id) = a_dsl::accounts
            .select(a_dsl::id)
            .filter(a_dsl::email.eq(&email))
            .first::<Uuid>(conn)
            .await
            .optional()
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?
        {
            Ok(Some(account_id))
        } else {
            Ok(None)
        }
    }

    async fn create_account(
        &self,
        conn: &mut (impl AsyncConnection<Backend = Pg> + Send),
        email: &str,
    ) -> Result<Uuid, tonic::Status> {
        use schema::accounts::dsl as a_dsl;

        let id = Uuid::new_v4();
        insert_into(a_dsl::accounts)
            .values(models::Account { id, email })
            .execute(conn)
            .await
            .map_err(|_| AuthError::AccountForEmailAreadyExists)?;

        Ok(id)
    }

    async fn get_or_create_account(
        &self,
        conn: &mut (impl AsyncConnection<Backend = Pg> + Send),
        email: &str,
    ) -> Result<Uuid, tonic::Status> {
        use schema::accounts::dsl as a_dsl;

        let account_id = if let Some(account) = self.get_account(conn, email).await? {
            account
        } else {
            let id = Uuid::new_v4();

            insert_into(a_dsl::accounts)
                .values(models::Account { id, email })
                .execute(conn)
                .await
                .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

            id
        };

        Ok(account_id)
    }

    async fn create_session(
        &self,
        conn: &mut (impl AsyncConnection<Backend = Pg> + Send),
        account_id: Uuid,
        password_login: bool,
    ) -> Result<SessionToken, tonic::Status> {
        let token: String = RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            (0..Self::SESSION_TOKEN_LEN)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect()
        });

        use schema::sessions::dsl as s_dsl;

        insert_into(s_dsl::sessions)
            .values(models::Session {
                token: token.as_ref(),
                account_id,
                password_login,
                expires_at: OffsetDateTime::now_utc().saturating_add(Self::SESSION_DURATION),
            })
            .execute(conn)
            .await
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

        Ok(SessionToken { token })
    }
}

#[tonic::async_trait]
impl lunu::auth::auth_server::Auth for Auth {
    async fn fetch_account(
        &self,
        request: tonic::Request<SessionToken>,
    ) -> Result<tonic::Response<OptionalAccount>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AuthError::PoolConnectionFailed)?;

        use schema::scopes;
        use schema::sessions;

        let (id, time) = sessions::dsl::sessions
            .select((sessions::dsl::account_id, sessions::dsl::expires_at))
            .filter(sessions::dsl::token.eq(&request.get_ref().token))
            .first::<(Uuid, OffsetDateTime)>(conn)
            .await
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

        if time < OffsetDateTime::now_utc() {
            use schema::sessions;

            delete(schema::sessions::dsl::sessions)
                .filter(sessions::dsl::token.eq(&request.get_ref().token))
                .execute(conn)
                .await
                .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

            Ok(tonic::Response::new(OptionalAccount { account: None }))
        } else {
            let scopes = sessions::dsl::sessions
                .inner_join(
                    scopes::dsl::scopes.on(scopes::dsl::account_id.eq(sessions::dsl::account_id)),
                )
                .filter(sessions::dsl::token.eq(&request.get_ref().token))
                .select(scopes::dsl::scope)
                .load::<models::ScopeKind>(conn)
                .await
                .map_err(|e| AuthError::QueryFailed(e.to_string()))?
                .into_iter()
                .map(|kind| kind as i32)
                .collect();

            Ok(tonic::Response::new(OptionalAccount {
                account: Some(Account {
                    id: id.to_string(),
                    scopes,
                }),
            }))
        }
    }

    async fn create_email_login_intent(
        &self,
        request: tonic::Request<AccountEmail>,
    ) -> Result<tonic::Response<EmailLoginIntent>, tonic::Status> {
        let mut conn = self
            .pool
            .get()
            .await
            .map_err(|_| AuthError::PoolConnectionFailed)?;
        let email = request.into_inner().email;

        let account_id = self.get_or_create_account(conn.deref_mut(), &email).await?;

        use schema::email_login_intents::dsl as eli_dsl;

        let pass_key: String = RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            (0..Self::SESSION_INTENT_CODE_LEN)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect()
        });
        let expires_at = OffsetDateTime::now_utc().saturating_add(Self::SESSION_INTENT_DURATION);
        let id = Uuid::new_v4();
        insert_into(eli_dsl::email_login_intents)
            .values(models::EmailLoginIntent {
                id,
                account_id,
                pass_key: pass_key.as_str(),
                expires_at,
            })
            .execute(&mut conn)
            .await
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

        let mut client = MAIL_CLIENT
            .get()
            .expect("MAIL_CLIENT used before it was initalized")
            .clone();
        client
            .send(Email {
                email,
                subject: "Pass key for lunu login".into(),
                body_html: format!("<h1>{pass_key}</h1>"),
            })
            .await?;

        Ok(tonic::Response::new(EmailLoginIntent {
            token: id.to_string(),
        }))
    }

    async fn login_with_email_login(
        &self,
        request: tonic::Request<EmailLoginParams>,
    ) -> Result<tonic::Response<SessionToken>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AuthError::PoolConnectionFailed)?;
        let resp = request.into_inner();
        let token = resp.token;
        let code = resp.code;

        use schema::email_login_intents::dsl as eli_dsl;

        let uuid = Uuid::from_str(&token).map_err(|_| AuthError::MalformedSessionToken)?;
        let session = eli_dsl::email_login_intents
            .select((eli_dsl::account_id, eli_dsl::pass_key, eli_dsl::expires_at))
            .filter(eli_dsl::id.eq(uuid))
            .load::<(Uuid, String, OffsetDateTime)>(conn)
            .await
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?
            .pop();

        if let Some((account_id, pass_key, expires_at)) = session {
            if expires_at < OffsetDateTime::now_utc() {
                Err(AuthError::SessionIntentTimeout.into())
            } else {
                if pass_key == code {
                    let token = self
                        .create_session(conn.deref_mut(), account_id, false)
                        .await?;

                    delete(eli_dsl::email_login_intents)
                        .filter(eli_dsl::id.eq(uuid))
                        .execute(conn)
                        .await
                        .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

                    Ok(tonic::Response::new(token))
                } else {
                    Err(AuthError::PasscodeDoesNotMatch.into())
                }
            }
        } else {
            Err(AuthError::BadSessionToken.into())
        }
    }

    async fn create_new_pass_login_intent(
        &self,
        request: tonic::Request<AccountEmail>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let AccountEmail { email } = request.into_inner();
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AuthError::PoolConnectionFailed)?;

        let account_id = self.get_or_create_account(conn.deref_mut(), &email).await?;

        use schema::new_pass_login_intents::dsl as fpli_dsl;

        let expires_at = OffsetDateTime::now_utc().saturating_add(Self::SESSION_INTENT_DURATION);
        let id: String = RNG.with(|rng| {
            let mut rng = rng.borrow_mut();
            (0..Self::NEW_PASS_LOGIN_TOKEN_LEN)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect()
        });
        insert_into(fpli_dsl::new_pass_login_intents)
            .values(models::NewPassLoginIntent {
                id: id.as_ref(),
                account_id,
                expires_at,
            })
            .execute(conn)
            .await
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

        let mut client = MAIL_CLIENT
            .get()
            .expect("MAIL_CLIENT used before it was initalized")
            .clone();
        client
            .send(Email {
                email,
                subject: "Login url token for lunu login".into(),
                body_html: format!("<h1>{id}</h1>"),
            })
            .await?;

        Ok(tonic::Response::new(Empty {}))
    }

    async fn login_with_new_pass_login(
        &self,
        request: tonic::Request<NewPassLoginParams>,
    ) -> Result<tonic::Response<SessionToken>, tonic::Status> {
        let NewPassLoginParams { token, password } = request.into_inner();
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AuthError::PoolConnectionFailed)?;

        use schema::new_pass_login_intents::dsl as fpli_dsl;
        let session = fpli_dsl::new_pass_login_intents
            .select((fpli_dsl::account_id, fpli_dsl::expires_at))
            .filter(fpli_dsl::id.eq(&token))
            .load::<(Uuid, OffsetDateTime)>(conn)
            .await
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?
            .pop();

        let Some((account_id, expires_at)) = session else {
            return Err(AuthError::BadSessionToken.into());
        };

        let now = OffsetDateTime::now_utc();
        if expires_at < now {
            delete(fpli_dsl::new_pass_login_intents)
                .filter(fpli_dsl::id.eq(&token))
                .execute(conn)
                .await
                .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

            Err(AuthError::BadSessionToken.into())
        } else {
            let salt = SaltString::generate(&mut OsRng);
            let argon = Argon2::default();

            let password_hash = argon
                .hash_password(password.as_bytes(), &salt)
                .map_err(|err| AuthError::PasswordHashingError(err))?
                .to_string();

            use schema::password_login::dsl as pl_dsl;

            // Delete an old password if it exists
            delete(pl_dsl::password_login)
                .filter(pl_dsl::account_id.eq(account_id))
                .execute(conn)
                .await
                .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

            insert_into(pl_dsl::password_login)
                .values(models::PasswordLogin {
                    account_id,
                    hash: &password_hash,
                    salt: salt.as_str(),
                    created_at: now,
                })
                .execute(conn)
                .await
                .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

            delete(fpli_dsl::new_pass_login_intents)
                .filter(fpli_dsl::id.eq(&token))
                .execute(conn)
                .await
                .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

            let token = self
                .create_session(conn.deref_mut(), account_id, true)
                .await?;

            Ok(tonic::Response::new(token))
        }
    }

    async fn create_with_password(
        &self,
        request: tonic::Request<PasswordParams>,
    ) -> Result<tonic::Response<SessionToken>, tonic::Status> {
        let PasswordParams { email, password } = request.into_inner();
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AuthError::PoolConnectionFailed)?;

        let account_id = self.create_account(conn.deref_mut(), &email).await?;

        let salt = SaltString::generate(&mut OsRng);
        let argon = Argon2::default();

        let password_hash = argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(|err| AuthError::PasswordHashingError(err))?
            .to_string();

        use schema::password_login::dsl as pl_dsl;

        insert_into(pl_dsl::password_login)
            .values(models::PasswordLogin {
                account_id,
                hash: &password_hash,
                salt: salt.as_str(),
                created_at: OffsetDateTime::now_utc(),
            })
            .execute(conn)
            .await
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

        let token = self
            .create_session(conn.deref_mut(), account_id, true)
            .await?;

        Ok(tonic::Response::new(token))
    }

    async fn login_with_password(
        &self,
        request: tonic::Request<PasswordParams>,
    ) -> Result<tonic::Response<SessionToken>, tonic::Status> {
        let PasswordParams { email, password } = request.into_inner();
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AuthError::PoolConnectionFailed)?;

        let account_id = self
            .get_account(conn.deref_mut(), &email)
            .await?
            .ok_or(AuthError::NoAccountForEmail(email))?;

        use schema::password_login::dsl as pl_dsl;
        let (hash, salt) = pl_dsl::password_login
            .select((pl_dsl::hash, pl_dsl::salt))
            .filter(pl_dsl::account_id.eq(account_id))
            .first::<(String, String)>(conn)
            .await
            .optional()
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?
            .ok_or(AuthError::AccountHasNoPasswordLogin)?;

        let salt = SaltString::from_b64(&salt).expect("Failed to parse the stored salt");
        let argon = Argon2::default();

        if hash
            == argon
                .hash_password(password.as_bytes(), &salt)
                .map_err(|err| AuthError::PasswordHashingError(err))?
                .to_string()
        {
            let token = self
                .create_session(conn.deref_mut(), account_id, true)
                .await?;

            Ok(tonic::Response::new(token))
        } else {
            Err(AuthError::WrongPassword.into())
        }
    }

    async fn cleanup_db(
        &self,
        _request: tonic::Request<Empty>,
    ) -> Result<tonic::Response<Empty>, tonic::Status> {
        let conn = &mut self
            .pool
            .get()
            .await
            .map_err(|_| AuthError::PoolConnectionFailed)?;
        let now = OffsetDateTime::now_utc();

        use schema::new_pass_login_intents::dsl as fpli_dsl;
        delete(fpli_dsl::new_pass_login_intents)
            .filter(fpli_dsl::expires_at.lt(now))
            .execute(conn)
            .await
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

        use schema::email_login_intents::dsl as eli_dsl;
        delete(eli_dsl::email_login_intents)
            .filter(eli_dsl::expires_at.lt(now))
            .execute(conn)
            .await
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

        use schema::sessions::dsl as s_dsl;
        delete(s_dsl::sessions)
            .filter(s_dsl::expires_at.lt(now))
            .execute(conn)
            .await
            .map_err(|e| AuthError::QueryFailed(e.to_string()))?;

        Ok(tonic::Response::new(Empty {}))
    }
}

enum AuthError {
    BadSessionToken,
    MalformedSessionToken,
    SessionIntentTimeout,
    PasscodeDoesNotMatch,
    QueryFailed(String),
    PasswordHashingError(argon2::password_hash::Error),
    PoolConnectionFailed,
    AccountForEmailAreadyExists,
    NoAccountForEmail(String),
    AccountHasNoPasswordLogin,
    WrongPassword,
}

impl From<AuthError> for tonic::Status {
    fn from(value: AuthError) -> Self {
        match value {
            AuthError::BadSessionToken => tonic::Status::invalid_argument(
                "Session token does not correspond to a session intent",
            ),
            AuthError::MalformedSessionToken => {
                tonic::Status::invalid_argument("Malformed session token")
            }
            AuthError::SessionIntentTimeout => {
                tonic::Status::resource_exhausted("This session token has already timed out")
            }
            AuthError::PasscodeDoesNotMatch => {
                tonic::Status::invalid_argument("Passcode does not match")
            }
            AuthError::QueryFailed(s) => tonic::Status::internal(format!("Query Failed: {s}")),
            AuthError::PasswordHashingError(err) => {
                tonic::Status::internal(format!("Failed to hash password: {err}"))
            }
            AuthError::PoolConnectionFailed => {
                tonic::Status::internal("Failed to connect to the internal pool")
            }
            AuthError::AccountForEmailAreadyExists => tonic::Status::invalid_argument(
                "Failed to create account for this email as one already exists",
            ),
            AuthError::NoAccountForEmail(email) => {
                tonic::Status::invalid_argument(format!("No account found for email: {email}"))
            }
            AuthError::AccountHasNoPasswordLogin => tonic::Status::invalid_argument(
                "Account has no password login, it only supports email login",
            ),
            AuthError::WrongPassword => {
                tonic::Status::invalid_argument("The password does not match the one on file")
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    init_clients().await;

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let config = AsyncDieselConnectionManager::<AsyncPgConnection>::new(&database_url);
    let auth = Auth {
        pool: Pool::builder().build(config).await?,
    };

    let addr = MICROSERVICE_ADDRS[&Microservice::Auth].parse()?;
    Server::builder()
        .add_service(AuthServer::new(auth))
        .serve(addr)
        .await?;

    Ok(())
}
