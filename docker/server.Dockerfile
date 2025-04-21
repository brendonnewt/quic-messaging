# Use official Rust image as a build environment
FROM rust:latest AS builder

# Set the working directory
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml ./

# Copy over the project files
COPY . ./

# Build the final application binary
RUN cargo build --release

# Use a minimal base image for the final image
FROM debian:bookworm-slim

# Install OpenSSL runtime (libssl.so.3) and CA certs, then clean up
RUN apt-get update \
&& apt-get install -y --no-install-recommends \
      libssl3 \
      ca-certificates \
&& rm -rf /var/lib/apt/lists/*

# Set the working directory inside the container
WORKDIR /app

# Copy the compiled application binary from the builder stage
COPY --from=builder /app/target/release/server /usr/local/bin/server

# ⬇️ Add this line to include the .env file
COPY --from=builder /app/.env .env

# Run the application when the container starts
CMD ["server"]
