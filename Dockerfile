# Build Stage
FROM --platform=linux/amd64 rust:1.79 as api_build

WORKDIR /app

# Copy all source files and the .env file
COPY . .
COPY .env .

# Build the Rust binary
RUN cargo build --release --bin server

# Final Stage
FROM --platform=linux/amd64 debian:stable-slim as api_final

WORKDIR /app

# Copy the Rust binary from the build stage
COPY --from=api_build /app/target/release/server .

# Copy the .env file to the final stage
COPY --from=api_build /app/.env .

# Install dependencies if required
RUN apt-get update && apt-get install -y libpq-dev

# Run the server
CMD ["./server"]
