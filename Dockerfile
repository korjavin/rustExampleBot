# --- Build Stage ---
# Use a specific Rust version for reproducibility
FROM rust:1.77 as builder

# Set working directory
WORKDIR /usr/src/app

# Install OS dependencies if needed (e.g., for openssl)
# RUN apt-get update &amp;&amp; apt-get install -y --no-install-recommends libssl-dev ca-certificates &amp;&amp; rm -rf /var/lib/apt/lists/*

# Copy manifests first to leverage Docker cache for dependencies
COPY Cargo.toml Cargo.lock ./
# Create a dummy src/main.rs to build dependencies only
RUN mkdir src && echo "fn main() {println!(\"if you see this, the build broke\")}" > src/main.rs
# Build only dependencies to cache this layer
RUN cargo build --release --locked
RUN rm -f target/release/deps/rustExampleBot* # Remove dummy build artifacts

# Copy the actual source code
COPY src ./src

# Build the final application binary, removing the dummy main.rs first
RUN rm -f src/main.rs
RUN cargo build --release --locked

# --- Runtime Stage ---
# Use a minimal base image
FROM debian:bullseye-slim as runtime

# Install necessary runtime dependencies (like ca-certificates for HTTPS)
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates &amp;&amp; rm -rf /var/lib/apt/lists/*

# Create a non-root user and group for security
RUN groupadd --system app && useradd --system --gid app app

# Set the working directory
WORKDIR /app

# Copy the compiled binary from the build stage
COPY --from=builder /usr/src/app/target/release/rustExampleBot /usr/local/bin/rustExampleBot

# Ensure the binary is executable
RUN chmod +x /usr/local/bin/rustExampleBot

# Switch to the non-root user
USER app

# Set the entrypoint for the container
CMD ["rustExampleBot"]