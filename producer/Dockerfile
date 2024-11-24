# Use an official Rust image
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Expose necessary ports (optional)
EXPOSE 30003

# Run the binary
CMD ["./target/release/producer"]
