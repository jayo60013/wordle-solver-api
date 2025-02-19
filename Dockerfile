# Use the official Rust image as a base image
FROM rust:latest as builder

# Set the working directory
WORKDIR /usr/src/wordle_solver_api

# Copy the Cargo.toml and Cargo.lock files to the working directory
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs file to pre-download dependencies
RUN mkdir src && \
    echo "fn main() {}" > src/main.rs

# Build the dependencies (this step will cache the dependencies)
RUN cargo build --release

# Remove the dummy main.rs file
RUN rm -f src/main.rs

# Copy the rest of the application code
COPY . .

# Build the application
RUN cargo build --release

# Use a slim runtime image
FROM debian:bookworm-slim

# Install necessary runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends openssl ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Set the working directory
WORKDIR /usr/src/wordle_solver_api

# Copy the binary from the builder stage
COPY --from=builder /usr/src/wordle_solver_api/target/release/wordle_solver .
COPY word_list.txt /usr/src/wordle_solver_api/word_list.txt

# Expose the port the app runs on
EXPOSE 5307

# Run the application
CMD ["./wordle_solver"]
