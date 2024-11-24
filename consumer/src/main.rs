use lapin::{options::*, types::FieldTable, Connection, ConnectionProperties};
use futures_util::stream::StreamExt;
use log::*;
use mongodb::{bson, bson::doc, options::ClientOptions, Client as MongoClient};
use redis::Commands;
use serde::{Deserialize, Serialize};
use simplelog::*;
use std::{env, fs};
use std::fs::{File, OpenOptions};
use dotenvy::dotenv;
use tokio;
use std::io::Write;

async fn setup_rabbitmq_consumer(
    amqp_url: &str,
    queue_name: &str,
    mongo_uri: &str,
    mongo_db: &str,
    redis_url: &str,
    log_file: &str,
) {
    // Set up logging
    CombinedLogger::init(vec![
        WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create(log_file).unwrap(),
        ),
    ])
        .unwrap();

    info!("Connecting to RabbitMQ...");
    let connection = Connection::connect(amqp_url, ConnectionProperties::default())
        .await
        .expect("Failed to connect to RabbitMQ");

    let channel = connection.create_channel().await.expect("Failed to open a channel");
    channel
        .queue_declare(
            queue_name,
            QueueDeclareOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to declare a queue");

    info!("Connecting to MongoDB...");
    let mongo_client = MongoClient::with_options(
        ClientOptions::parse(mongo_uri).await.expect("Invalid MongoDB URI"),
    )
        .expect("Failed to connect to MongoDB");
    let mongo_collection = mongo_client
        .database(mongo_db)
        .collection::<BaseStationMessage>("messages");

    info!("Connecting to Redis...");
    let redis_client =
        redis::Client::open(redis_url).expect("Failed to connect to Redis");
    let mut redis_conn = redis_client.get_connection().expect("Failed to get Redis connection");

    info!("Starting to consume messages...");
    let mut consumer = channel
        .basic_consume(
            queue_name,
            "consumer_tag",
            BasicConsumeOptions::default(),
            FieldTable::default(),
        )
        .await
        .expect("Failed to start consuming");

    while let Some(delivery) = consumer.next().await {
        if let Ok((_, delivery)) = delivery {
            let message = String::from_utf8_lossy(&delivery.data);
            info!("Received message: {}", message);

            // Deserialize message
            let message_data: BaseStationMessage     =
                parse_socket_data(&message).expect("Failed to parse message");

            // Split the generated_date into parts
            let date_parts: Vec<&str> = message_data.generated_date.split('/').collect();
            if date_parts.len() != 3 {
                eprintln!("Invalid generated_date format: {}", message_data.generated_date);
                return;
            }
            // Extract parts
            let year = date_parts[0];
            let month = date_parts[1];
            let day = date_parts[2];
            let hex_code = message_data.hex_ident.clone().unwrap_or_default();

            // Define the hierarchical path
            let hierarchical_path = format!("{}.{}.{}.{}", year, month, day, hex_code);

            // Convert the message to BSON format
            let bson_data = bson::to_bson(&message_data).unwrap_or_default();

            // Use the `$push` operator to append data to the array at the specified path
            let update_doc = doc! {
                "$push": {
                    &hierarchical_path: bson_data
                }
            };
            // Perform the update with upsert
            mongo_collection
                .update_one(
                    doc! {}, // Match all documents (can be refined if necessary)
                    update_doc,
                    mongodb::options::UpdateOptions::builder().upsert(true).build(),
                )
                .await
                .expect("Failed to insert or update document");
            
            // Write message to a log file
            let log_dir = format!("/app/logs/{}", message_data.generated_date);
            let file_name = format!("{}.log", message_data.generated_date.replace("/", "-"));
            let log_path = format!("{}/{}", log_dir, file_name);

            // Create the directories recursively
            if let Err(e) = fs::create_dir_all(log_dir.clone()) {
                eprintln!("Failed to create log directory {}: {}", log_dir, e);
                return;
            }

            // Open the file in append mode and write the message
            let mut file = OpenOptions::new()
                .create(true)
                .append(true)
                .open(&log_path)
                .expect("Failed to open log file");
            writeln!(file, "{}", message).expect("Failed to write to log file");

            // Store in Redis as lastMessage
            redis_conn.set::<_, _, ()>("lastMessage", message.as_ref())
                .expect("Failed to set value in Redis");

            // Acknowledge the message
            delivery
                .ack(BasicAckOptions::default())
                .await
                .expect("Failed to acknowledge message");
        }
    }
}

#[tokio::main]
async fn main() {
    // Check if running in Docker
    let running_in_docker = env::var("RUNNING_IN_DOCKER").is_ok();

    // Load .env only if not running in Docker
    if !running_in_docker {
        dotenv().ok();
        println!("Loaded environment variables from .env file");
    } else {
        println!("Running in Docker, using Docker environment variables");
    }

    // Read environment variables
    let amqp_url = format!(
        "amqp://{}:{}@{}:5672",
        env::var("RABBITMQ_USER").expect("RABBIT_MQ_USER not set"),
        env::var("RABBITMQ_PASS").expect("RABBIT_MQ_PASS not set"),
        env::var("MESSAGE_QUEUE_HOST").expect("MESSAGE_QUEUE_HOST not set")
    );
    let queue_name = env::var("QUEUE_NAME").expect("QUEUE_NAME not set");
    let mongo_uri = format!(
        "mongodb://{}:27017",
        env::var("NOSQL_DB_HOST").expect("NOSQL_DB_HOST not set")
    );
    let mongo_db = env::var("NOSQL_DB_NAME").expect("NOSQL_DB_NAME not set");
    let redis_url = format!(
        "redis://{}",
        env::var("REDIS_HOST").expect("REDIS_HOST not set")
    );
    let log_file = env::var("LOG_FILE").expect("LOG_FILE not set");

    setup_rabbitmq_consumer(
        &amqp_url,
        &queue_name,
        &mongo_uri,
        &mongo_db,
        &redis_url,
        &log_file,
    )
        .await;
}


/// Represents a BaseStation message structure
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BaseStationMessage {
    pub message_type: String,   // Message type (e.g., MSG, STA)
    pub transmission_type: Option<u8>, // Transmission type (1-8 for MSG)
    pub session_id: Option<u32>,
    pub aircraft_id: Option<u32>,
    pub hex_ident: Option<String>,
    pub flight_id: Option<u32>,
    pub generated_date: String, // Format: YYYY/MM/DD
    pub generated_time: String, // Format: HH:MM:SS.sss
    pub logged_date: String,    // Format: YYYY/MM/DD
    pub logged_time: String,    // Format: HH:MM:SS.sss
    pub callsign: Option<String>,
    pub altitude: Option<i32>,
    pub ground_speed: Option<f64>,
    pub track: Option<f64>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub vertical_rate: Option<i32>,
    pub squawk: Option<String>,
    pub alert: Option<bool>,
    pub emergency: Option<bool>,
    pub spi: Option<bool>,
    pub is_on_ground: Option<bool>,
}

/// Represents a BST file record
#[derive(Debug, Serialize, Deserialize)]
pub struct BaseStationRecord {
    pub date: String,           // Format: YYYY/MM/DD
    pub time: String,           // Format: HH:MM:SS.sss
    pub unique_id: String,
    pub hex_ident: String,
    pub callsign: String,
    pub country: String,
    pub unknown_field: String,  // Placeholder for the unclassified fields
    pub altitude: i32,
    pub pressure_altitude: i32,
    pub latitude: f64,
    pub longitude: f64,
    pub vertical_rate: i32,
    pub heading: f64,
    pub ground_speed: f64,
    pub track: f64,
    pub squawk: String,
    pub alert_flag: bool,
}

/// Parses a raw socket data line into a `BaseStationMessage`
fn parse_socket_data(line: &str) -> Result<BaseStationMessage, serde_json::Error> {
    let fields: Vec<&str> = line.split(',').collect();
    let message = BaseStationMessage {
        message_type: fields.get(0).unwrap_or(&"").to_string(),
        transmission_type: fields.get(1).and_then(|v| v.parse().ok()),
        session_id: fields.get(2).and_then(|v| v.parse().ok()),
        aircraft_id: fields.get(3).and_then(|v| v.parse().ok()),
        hex_ident: fields.get(4).map(|s| s.to_string()),
        flight_id: fields.get(5).and_then(|v| v.parse().ok()),
        generated_date: fields.get(6).unwrap_or(&"").to_string(),
        generated_time: fields.get(7).unwrap_or(&"").to_string(),
        logged_date: fields.get(8).unwrap_or(&"").to_string(),
        logged_time: fields.get(9).unwrap_or(&"").to_string(),
        callsign: fields.get(10).map(|s| s.to_string()),
        altitude: fields.get(11).and_then(|v| v.parse().ok()),
        ground_speed: fields.get(12).and_then(|v| v.parse().ok()),
        track: fields.get(13).and_then(|v| v.parse().ok()),
        latitude: fields.get(14).and_then(|v| v.parse().ok()),
        longitude: fields.get(15).and_then(|v| v.parse().ok()),
        vertical_rate: fields.get(16).and_then(|v| v.parse().ok()),
        squawk: fields.get(17).map(|s| s.to_string()),
        alert: fields.get(18).and_then(|v| match v {
            &"-1" => Some(true),
            &"0" => Some(false),
            _ => None,
        }),
        emergency: fields.get(19).and_then(|v| match v {
            &"-1" => Some(true),
            &"0" => Some(false),
            _ => None,
        }),
        spi: fields.get(20).and_then(|v| match v {
            &"-1" => Some(true),
            &"0" => Some(false),
            _ => None,
        }),
        is_on_ground: fields.get(21).and_then(|v| match v {
            &"-1" => Some(true),
            &"0" => Some(false),
            _ => None,
        }),
    };
    Ok(message)
}

/// Parses a BST file record into a `BaseStationRecord`
fn parse_bst_record(line: &str) -> Result<BaseStationRecord, serde_json::Error> {
    let fields: Vec<&str> = line.split(',').collect();
    let record = BaseStationRecord {
        date: fields[0].to_string(),
        time: fields[1].to_string(),
        unique_id: fields[2].to_string(),
        hex_ident: fields[3].to_string(),
        callsign: fields[4].to_string(),
        country: fields[5].to_string(),
        unknown_field: fields[6].to_string(),
        altitude: fields[7].parse().unwrap_or(0),
        pressure_altitude: fields[8].parse().unwrap_or(0),
        latitude: fields[9].parse().unwrap_or(0.0),
        longitude: fields[10].parse().unwrap_or(0.0),
        vertical_rate: fields[11].parse().unwrap_or(0),
        heading: fields[12].parse().unwrap_or(0.0),
        ground_speed: fields[13].parse().unwrap_or(0.0),
        track: fields[14].parse().unwrap_or(0.0),
        squawk: fields[15].to_string(),
        alert_flag: fields[16] == "-1",
    };
    Ok(record)
}

/// Converts a `BaseStationMessage` to a JSON string
fn write_json_message(message: &BaseStationMessage) -> String {
    serde_json::to_string_pretty(message).unwrap()
}

/// Converts a `BaseStationRecord` to a JSON string
fn write_json_record(record: &BaseStationRecord) -> String {
    serde_json::to_string_pretty(record).unwrap()
}
