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
    position_timestamp: DateTime<Utc>,
    position_created_at: DateTime<Utc>,
    battery_level: Option<i32>,
    battery_voltage: Option<f32>,
    telemetry_timestamp: Option<DateTime<Utc>>,
    telemetry_created_at: Option<DateTime<Utc>>,
}

#[derive(Serialize, Debug)]
struct UserData {
    user_id: String,
    latitude: BigDecimal,
    longitude: BigDecimal,
    battery_level: String,
    battery_voltage: String,
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
        battery_level: raw
            .battery_level
            .map(|l| format!("{l}"))
            .unwrap_or("null".to_string()),
        battery_voltage: raw
            .battery_voltage
            .map(|l| format!("{l}"))
            .unwrap_or("null".to_string()),
        timestamp: raw.position_timestamp,
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
        "
        WITH LatestPositions AS ( 
            SELECT user_id, MAX(timestamp) AS max_timestamp
            FROM positions
            WHERE ST_Y(location) != 0 OR ST_X(location) != 0
            GROUP BY user_id
        ),
        LatestTelemetry AS (
            SELECT user_id, MAX(timestamp) AS max_timestamp
            FROM telemetry
            GROUP BY user_id
        ),
        LatestData AS (
            SELECT
                lp.user_id,
                ROUND(ST_X(p.location)::numeric, 5) as longitude,
                ROUND(ST_Y(p.location)::numeric, 5) as latitude,
                p.timestamp AS position_timestamp,
                p.created_at AS position_created_at,
                t.battery_level,
                t.voltage AS battery_voltage,
                t.timestamp AS telemetry_timestamp,
                t.created_at AS telemetry_created_at
            FROM LatestPositions lp
            JOIN positions p ON lp.user_id = p.user_id AND lp.max_timestamp = p.timestamp
            LEFT JOIN LatestTelemetry lt ON lp.user_id = lt.user_id
            LEFT JOIN telemetry t ON lt.user_id = t.user_id AND lt.max_timestamp = t.timestamp
            ORDER BY lp.user_id DESC
        )
        SELECT * FROM LatestData;
        ",
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
