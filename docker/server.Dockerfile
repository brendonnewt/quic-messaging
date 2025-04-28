# Use official Rust image as a build environment
FROM rust:latest AS builder

# Set the working directory
WORKDIR /app

# Install musl-tools for musl-gcc (needed for ring, rustls, etc.)
RUN apt-get update && apt-get install -y musl-tools

# Copy the entire project context (including server/ and shared/)
COPY server server
COPY shared shared
COPY client client
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock

# Install MUSL target for static linking
RUN rustup target add x86_64-unknown-linux-musl

# Build the server crate
WORKDIR /app/server
RUN cargo build --release --target x86_64-unknown-linux-musl --bin server

RUN find /app -type f

FROM scratch

# Set working directory
WORKDIR /app

# Copy the compiled static binary into final image
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/server /server

# Expose the server port
EXPOSE 8080

# Run the static server binary
CMD ["/server"]
