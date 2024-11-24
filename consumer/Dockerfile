# Use an official Rust image
FROM rust:latest

# Set the working directory
WORKDIR /app

# Copy the source code
COPY . .

# Build the application
RUN cargo build --release

# Expose no specific port (this is a background worker)
EXPOSE 0

# Run the binary
CMD ["./target/release/consumer"]
