FROM rust:1.82-slim AS builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create appuser
RUN adduser --no-create-home --disabled-password --gecos "" appuser

WORKDIR /app
COPY . .

# Build with optimizations
RUN cargo build --release

# Generate SBOM
RUN cargo install cargo-cyclonedx
RUN cargo cyclonedx -f json --override-filename bom

# Security scan stage
FROM aquasec/trivy:latest AS security-scan
COPY --from=builder /app /app
RUN trivy fs --severity HIGH,CRITICAL --exit-code 1 /app

# Final stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy binary, SBOM and user info
COPY --from=builder /app/target/release/acci_base /app/acci_base
COPY --from=builder /app/bom.json /app/bom.json
COPY --from=builder /etc/passwd /etc/passwd

# Use non-root user
USER appuser

# Set secure defaults
ENV RUST_BACKTRACE=0
ENV RUST_LOG=info

EXPOSE 8080

ENTRYPOINT ["/app/acci_base"]
