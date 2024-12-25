# Build Stage
FROM --platform=linux/amd64 rust:1.79 as api_build

WORKDIR /app

# Install required build dependencies (optional if needed for specific crates)
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev build-essential && \
    apt-get clean && rm -rf /var/lib/apt/lists/*

# Copy all source files and the .env file
COPY . .
COPY .env .

# Copy the templates directory
COPY src/templates ./src/templates

# Build the Rust binary
RUN cargo build --release --bin server

# Final Stage
FROM --platform=linux/amd64 debian:stable-slim as api_final

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    git \
    && apt-get clean && rm -rf /var/lib/apt/lists/*

# Copy the Rust binary from the build stage
COPY --from=api_build /app/target/release/server .

# Copy the .env file to the final stage
COPY --from=api_build /app/.env .

# Copy the templates directory to the final stage
COPY --from=api_build /app/src/templates ./src/templates

# Ensure the binary has execution permissions
RUN chmod +x /app/server

# Expose the port for the application
EXPOSE 5001

# Run the server
CMD ["./server"]
