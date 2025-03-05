# Build Stage
FROM rustlang/rust:nightly as api_build

WORKDIR /app

# Copy all source files and the .env file
COPY . .
COPY .env .

# Copy the templates directory
COPY src/templates ./src/templates

# Build the Rust binary
RUN cargo build --release --bin server

# Final Stage
FROM debian:bookworm-slim as api_final

WORKDIR /app

# Install psql runtime library, OpenSSL 3, and CA certificates
RUN apt-get update && apt-get install -y \
    libpq-dev ca-certificates &&\
    rm -rf /var/lib/apt/lists/*

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
