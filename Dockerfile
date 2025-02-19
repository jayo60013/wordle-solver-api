# Use Rust official image as the base for building
FROM rust:1.76-alpine AS builder
WORKDIR /app
RUN apk add --no-cache pkgconfig libssl3-dev

# Copy the Cargo files first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create a dummy file to leverage caching for dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
COPY src ./src
RUN cargo build --release

FROM debian:bullseye-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/wordle-solver-api /app/wordle-solver-api
EXPOSE 5307
CMD ["/app/wordle-solver-api"]
