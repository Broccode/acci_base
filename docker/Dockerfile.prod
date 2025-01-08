FROM --platform=$BUILDPLATFORM rust:1.82-slim AS builder

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
ARG TARGETPLATFORM
RUN case "$TARGETPLATFORM" in \
    "linux/amd64")  TARGET="x86_64-unknown-linux-gnu" ;; \
    "linux/arm64")  TARGET="aarch64-unknown-linux-gnu" ;; \
    "linux/ppc64le")  TARGET="powerpc64le-unknown-linux-gnu" ;; \
    *)  TARGET="x86_64-unknown-linux-gnu" ;; \
    esac && \
    rustup target add $TARGET && \
    cargo build --release --target $TARGET

# Generate SBOM
RUN cargo install cargo-cyclonedx
RUN cargo cyclonedx -f json --override-filename bom

# Security scan stage
FROM aquasec/trivy:latest AS security-scan
COPY --from=builder /app /app
RUN trivy fs --severity HIGH,CRITICAL --exit-code 1 /app

# Final stage
FROM --platform=$TARGETPLATFORM gcr.io/distroless/cc-debian12:nonroot

# Copy binary, SBOM and user info
COPY --from=builder /app/target/*/release/acci_base /app/acci_base
COPY --from=builder /app/bom.json /app/bom.json
COPY --from=builder /etc/passwd /etc/passwd

# Use non-root user
USER appuser

# Set secure defaults
ENV RUST_BACKTRACE=0
ENV RUST_LOG=info

EXPOSE 8080

ENTRYPOINT ["/app/acci_base"]
