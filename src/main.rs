use std::fs;

use bigdecimal::BigDecimal;
use chrono::DateTime;
use chrono::Utc;
use dotenvy::dotenv;
use once_cell::sync::Lazy;
use serde::Serialize;
use sqlx::postgres::PgConnectOptions;
use sqlx::postgres::PgPoolOptions;
use sqlx::FromRow;
use tera::Context;
use tera::Tera;

pub static TEMPLATES: Lazy<Tera> = Lazy::new(|| match Tera::new("templates/**/*") {
    Ok(t) => t,
    Err(e) => {
        println!("Parsing error(s): {}", e);
        ::std::process::exit(1);
    }
});

#[derive(FromRow)]
struct RawUserData {
    user_id: String,
    latitude: sqlx::types::BigDecimal,
    longitude: sqlx::types::BigDecimal,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
struct UserData {
    user_id: String,
    latitude: BigDecimal,
    longitude: BigDecimal,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
struct IndexContext {
    users: Vec<UserData>,
}

fn raw_to_row(raw: RawUserData) -> UserData {
    UserData {
        user_id: raw.user_id,
        latitude: raw.latitude,
        longitude: raw.longitude,
        timestamp: raw.timestamp,
    }
}

static CONFIG: Lazy<PgConnectOptions> = Lazy::new(|| {
    dotenv().ok();

    let postgres_database = std::env::var("POSTGRES_DATABASE").unwrap();
    let postgres_host = std::env::var("POSTGRES_HOST").unwrap();
    let postgres_password = std::env::var("POSTGRES_PASSWORD").unwrap();
    let postgres_port = std::env::var("POSTGRES_PORT").unwrap();
    let postgres_user = std::env::var("POSTGRES_USER").unwrap();

    PgConnectOptions::new()
        .username(&postgres_user)
        .password(&postgres_password)
        .host(&postgres_host)
        .port(postgres_port.parse::<u16>().unwrap())
        .database(&postgres_database)
});

#[tokio::main]
async fn main() {
    let pool = PgPoolOptions::new()
        .max_connections(5) // Set the maximum number of connections in the pool
        .connect_with(CONFIG.clone())
        .await
        .unwrap();

    let positions: Vec<UserData> = sqlx::query_as::<_, RawUserData>(
        "SELECT \
        DISTINCT ON (user_id) \
            user_id, \
            ROUND(ST_Y(location)::numeric, 5) as latitude, \
            ROUND(ST_X(location)::numeric, 5) as longitude, \
            timestamp FROM positions \
        WHERE ST_Y(location) != 0 OR ST_X(location) != 0 \
        ORDER BY user_id, timestamp DESC",
    )
    .fetch_all(&pool)
    .await
    .unwrap()
    .into_iter()
    .map(&raw_to_row)
    .collect();

    let context: IndexContext = IndexContext { users: positions };

    let output = TEMPLATES
        .render("index.html", &Context::from_serialize(context).unwrap())
        .unwrap();

    let file = "index.html";

    fs::write(file, output).unwrap();
}
