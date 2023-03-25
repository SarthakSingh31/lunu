pub mod models;
pub mod schema;

// use std::env;

// use diesel::pg::PgConnection;
// use diesel::prelude::*;
// use dotenvy::dotenv;

// fn main() {
//     dotenv().ok();

//     let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
//     let mut conn = PgConnection::establish(&database_url)
//         .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

//     // schema::accounts::dsl::accounts.load(&mut conn).unwrap();
// }
