# Build Stage
FROM rustlang/rust:nightly AS api_build

WORKDIR /app

# Copy all source files and the .env file
COPY . .
COPY .env .

# Copy the templates directory
COPY src/templates ./src/templates

# Build the Rust binary
RUN cargo build --release --bin server

# Final Stage
FROM debian:bookworm-slim AS api_final

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    libpq-dev ca-certificates curl git-all cmake gcc libssl-dev pkg-config libclang-dev build-essential && \
    rm -rf /var/lib/apt/lists/*

# Install Rust non-interactively and set PATH
RUN curl https://sh.rustup.rs -sSf | bash -s -- -y && \
    echo 'export PATH="/root/.cargo/bin:$PATH"' >> /root/.bashrc && \
    export PATH="/root/.cargo/bin:$PATH"

# Use full path for Cargo since PATH may not persist across layers
RUN /root/.cargo/bin/cargo install --locked --git https://github.com/MystenLabs/sui.git --branch testnet --features tracing sui

COPY client.yaml /root/.sui/sui_config/client.yaml
COPY sui.keystore /root/.sui/sui_config/sui.keystore

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
