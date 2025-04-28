# Use official Rust image as a build environment
FROM rust:latest AS builder

# Set the working directory
WORKDIR /app/server

# Copy over the project files
COPY . .

# Build the final application binary
RUN cargo build --release

# Use a minimal base image for the final image
FROM debian:bookworm-slim

# Install necessary dependencies
RUN apt-get update

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled application binary from the builder stage
COPY --from=builder /app/server/target/release/server /usr/local/bin/server

# Run the application when the container starts
CMD ["server"]
