# ADS-B Data Processing System

This project is an ADS-B data processing system that captures, processes, and stores ADS-B data using a microservices architecture. The system consists of multiple services including a producer, consumer, message queue, Redis, and MongoDB.

## Table of Contents

- [Architecture](#architecture)
- [Services](#services)
- [Environment Variables](#environment-variables)
- [Getting Started](#getting-started)
- [Building and Running](#building-and-running)
- [License](#license)

## Architecture

The system is composed of the following services:

- **Producer**: Captures ADS-B data and publishes it to a RabbitMQ queue.
- **Consumer**: Consumes ADS-B data from the RabbitMQ queue, processes it, and stores it in MongoDB and Redis.
- **RabbitMQ**: Message broker for communication between producer and consumer.
- **Redis**: In-memory data structure store used for caching.
- **MongoDB**: NoSQL database for storing historical ADS-B data.

## Services

### Producer

The producer service captures ADS-B data from a specified server and publishes it to a RabbitMQ queue.

### Consumer

The consumer service consumes ADS-B data from the RabbitMQ queue, processes it, and stores it in MongoDB and Redis.

### RabbitMQ

RabbitMQ is used as the message broker for communication between the producer and consumer services.

### Redis

Redis is used for caching the last message received by the consumer.

### MongoDB

MongoDB is used to store historical ADS-B data.

## Environment Variables

The following environment variables are used in the project:

- `ADSB_SERVER`: The ADS-B server address.
- `PUBLISH_INTERVAL`: Interval in milliseconds for publishing messages.
- `LOG_FILE`: Path to the log file.
- `REDIS_HOST`: Redis server address.
- `HTTP_PORT`: HTTP server port.
- `MESSAGE_QUEUE_HOST`: RabbitMQ server address.
- `QUEUE_NAME`: Name of the RabbitMQ queue.
- `NOSQL_DB_HOST`: MongoDB server address.
- `NOSQL_DB_NAME`: Name of the MongoDB database.
- `RABBITMQ_USER`: RabbitMQ username.
- `RABBITMQ_PASS`: RabbitMQ password.

## Getting Started

### Prerequisites

- Docker
- Docker Compose
- Rust

### Clone the Repository

```sh
git clone https://github.com/saviobatista/adsb.git
cd adsb
```

### Set Up Environment Variables

Copy the .env.sample file to .env and update the values as needed.

```sh
cp .env.sample .env
```

### Building and Running

#### Using Docker Compose

Build and start the services using Docker Compose:
    
```sh
docker-compose up --build
```

#### Building Manually

Build the producer and consumer services manually:

```sh
cd producer
cargo build --release
cd ../consumer
cargo build --release
```

Run the producer and consumer services:

```sh
./producer/target/release/producer
./consumer/target/release/consumer
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
