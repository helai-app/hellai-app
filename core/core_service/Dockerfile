# Build stage: Use Rust for building the application
FROM rust:1.82 AS builder

# Set the working directory
WORKDIR /usr/src/core_service

# Install necessary libraries, including protobuf compiler
RUN apt-get update && \
    apt-get install -y protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

# Copy Cargo.toml and source files to optimize Docker cache layers
COPY Cargo.toml .
COPY .env .
COPY api/ api/
COPY core_database/ core_database/
COPY core_debugger/ core_debugger/
COPY core_error/ core_error/
COPY migration/ migration/
COPY service/ service/
COPY src/ src/

# Build the application in release mode
RUN cargo build --release

RUN echo "JWT_SECRET_KEY=\nPASSWORD_SECRET_KEY=\nDATABASE_URL=postgres://core_service_db_admin:123QWEasd@db:5432/core_service_db\nLOG_ALL_EVENTS=false" > /usr/src/core_service/.env

# Runtime stage: Create a minimal image for running the application
FROM ubuntu:24.10

# Set environment variables and install necessary libraries
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && \
    apt-get install -y ca-certificates libssl-dev libssl3 openssl && \
    rm -rf /var/lib/apt/lists/*

# Copy the built application from the builder stage
COPY --from=builder /usr/src/core_service/target/release/core_service /usr/local/bin/
COPY --from=builder /usr/src/core_service/.env /

# Expose the application port
EXPOSE 50052

# Command to run the server
CMD ["core_service"]
