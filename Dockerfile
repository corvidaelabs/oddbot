# Build stage
FROM rust:1.85-slim-bullseye as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create a new empty shell project
WORKDIR /app

# Copy workspace configuration
COPY Cargo.toml Cargo.lock ./

# Copy all workspace members
COPY labs/discord-bot ./labs/discord-bot

# Build the project for release
RUN cargo build --release -p discord-bot

# Runtime stage
FROM debian:bullseye-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl1.1 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy the binary from builder
COPY --from=builder /app/target/release/discord-bot /usr/local/bin/oddlaws-bot

# Expose the default port
EXPOSE 3000

# # Set the entrypoint
ENTRYPOINT ["/usr/local/bin/oddlaws-bot"]
