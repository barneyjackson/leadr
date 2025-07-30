# Build stage
FROM rust:slim-bullseye AS builder

WORKDIR /app

# Install necessary dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency files first for better layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached if Cargo.toml doesn't change)
RUN cargo build --release
RUN rm src/main.rs

# Copy source code
COPY src ./src
COPY migrations ./migrations
# Copy SQLx offline query data for compilation
COPY .sqlx ./.sqlx

# Build the application
# Remove the dummy build artifacts and build the real application
RUN rm ./target/release/deps/leadr_api*
# Use SQLx offline mode for compilation
ENV SQLX_OFFLINE=true
RUN cargo build --release

# Runtime stage
FROM debian:bullseye-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create a non-root user
RUN useradd -m -u 1001 appuser

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/leadr-api ./leadr-api
COPY --from=builder /app/migrations ./migrations

# Change ownership to appuser
RUN chown -R appuser:appuser /app

# Switch to non-root user
USER appuser

# Expose the port the app runs on
EXPOSE 3000

# Run the binary
CMD ["./leadr-api"]