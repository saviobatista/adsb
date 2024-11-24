use lapin::{
    options::BasicPublishOptions, BasicProperties, Connection, ConnectionProperties, Result,
};
use lapin::options::ExchangeDeclareOptions;
use lapin::types::FieldTable;
use tokio::runtime::Runtime;

pub fn publish_message(queue_host: &str, queue_name: &str, message: &str, username: &str, password: &str) -> Result<()> {
    let uri = format!("amqp://{}:{}@{}:5672", username, password, queue_host);

    // Use a Tokio runtime to execute async code
    let rt = Runtime::new().unwrap();

    rt.block_on(async {
        // Connect to RabbitMQ
        let conn = Connection::connect(&uri, ConnectionProperties::default()).await?;
        let channel = conn.create_channel().await?;

        // Publish the message
        channel
            .basic_publish(
                "",
                queue_name,
                BasicPublishOptions::default(),
                message.as_bytes(),
                BasicProperties::default(),
            )
            .await?;

        println!("Message published: {}", message);

        Ok(())
    })
}
