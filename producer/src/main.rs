mod adsb_handler;
mod queue;

use adsb_handler::capture_adsb_data;
use queue::publish_message;

use std::{env, thread, time::Duration};

fn main() {
    // Load configuration
    let queue_host = env::var("MESSAGE_QUEUE_HOST").unwrap_or_else(|_| "localhost".to_string());
    let queue_name = env::var("QUEUE_NAME").unwrap_or_else(|_| "adsb_data".to_string());
    let queue_user = env::var("RABBITMQ_USER").unwrap_or_else(|_| "user".to_string());
    let queue_pass = env::var("RABBITMQ_PASS").unwrap_or_else(|_| "password".to_string());
    let interval = env::var("PUBLISH_INTERVAL")
        .unwrap_or_else(|_| "1000".to_string())
        .parse::<u64>()
        .unwrap_or(1000);

    println!(
        "Producer started. Sending data to queue '{}' at '{}'.",
        queue_name, queue_host
    );

    loop {
        // Capture ADS-B data
        match capture_adsb_data() {
            Some(data) => {
                // Publish to RabbitMQ
                if let Err(e) = publish_message(&queue_host, &queue_name, &data, &queue_user, &queue_pass) {
                    eprintln!("Failed to publish message: {}", e);
                } else {
                    println!("Published message: {}", data);
                }
            }
            None => {
                eprintln!("No data captured.");
            }
        }

        // Wait before sending the next message
        thread::sleep(Duration::from_millis(interval));
    }
}
