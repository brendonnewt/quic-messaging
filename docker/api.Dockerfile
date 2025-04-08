# Use official Rust image as a build environment
FROM rust:latest AS builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY ../api/Cargo.toml ../api/Cargo.lock ./

# Copy over the project files
COPY ../api ./

# Build the final application binary
RUN cargo build --release

# Use a minimal base image for the final image
FROM debian:bookworm-slim

# Install necessary dependencies
RUN apt-get update

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled application binary from the builder stage
COPY --from=builder /app/target/release/api /usr/local/bin/api

# Copy the .env file into the container if needed
#COPY .env /app/.env

# Run the application when the container starts
CMD ["api"]