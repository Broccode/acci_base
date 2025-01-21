# Docker Build Process

## Overview

The ACCI Framework uses a modern, multi-platform Docker build process with the following features:
- Native multi-platform builds using Docker BuildKit
- Multi-stage builds for optimized image size
- Build caching for faster builds
- Security scanning integration
- Container registry support
- Distroless base images

## Build Process

### Multi-Platform Support

Currently supported platforms:
- linux/amd64
- linux/ppc64le

### Build Stages

```dockerfile
# Build Stage
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime Stage
FROM gcr.io/distroless/cc
COPY --from=builder /app/target/release/acci /
ENTRYPOINT ["/acci"]
```

### BuildKit Features

```bash
# Enable BuildKit
export DOCKER_BUILDKIT=1

# Build multi-platform image
docker buildx build \
  --platform linux/amd64,linux/ppc64le \
  --cache-from type=registry,ref=user/app:cache \
  --cache-to type=registry,ref=user/app:cache \
  --tag user/app:latest \
  .
```

## Development Environment

### Dockerfile.dev
```dockerfile
FROM rust:1.75

# Development tools
RUN rustup component add rustfmt clippy
RUN cargo install cargo-watch cargo-audit

WORKDIR /app
COPY . .

# Development command
CMD ["cargo", "watch", "-x", "run"]
```

### docker-compose.dev.yml
```yaml
version: '3.8'
services:
  app:
    build:
      context: .
      dockerfile: Dockerfile.dev
    volumes:
      - .:/app
      - cargo-cache:/usr/local/cargo/registry
    environment:
      - RUN_MODE=dev
      - APP__SERVER__BACKEND_PORT=3333
    ports:
      - "3333:3333"
    depends_on:
      - db
      - redis

  db:
    image: postgres:15
    environment:
      - POSTGRES_USER=acci
      - POSTGRES_PASSWORD=acci
      - POSTGRES_DB=acci_dev

  redis:
    image: redis:7
    ports:
      - "6379:6379"

volumes:
  cargo-cache:
```

## Production Environment

### Dockerfile
```dockerfile
# Build Stage
FROM rust:1.75 as builder

# Create appuser
ENV USER=acci
ENV UID=10001
RUN adduser \
    --disabled-password \
    --gecos "" \
    --home "/nonexistent" \
    --shell "/sbin/nologin" \
    --no-create-home \
    --uid "${UID}" \
    "${USER}"

WORKDIR /app

# Build dependencies - create a dummy main.rs to cache dependencies
RUN mkdir src
RUN echo "fn main() {}" > src/main.rs
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release
RUN rm -rf src

# Build application
COPY . .
RUN cargo build --release

# Runtime Stage
FROM gcr.io/distroless/cc

# Import from builder
COPY --from=builder /etc/passwd /etc/passwd
COPY --from=builder /etc/group /etc/group
COPY --from=builder /app/target/release/acci /app/acci

WORKDIR /app

# Use an unprivileged user
USER acci:acci

# Run the application
CMD ["/app/acci"]
```

### docker-compose.yml
```yaml
version: '3.8'
services:
  app:
    image: acci-framework:latest
    environment:
      - RUN_MODE=prod
      - APP__SERVER__BACKEND_PORT=8080
    ports:
      - "8080:8080"
    depends_on:
      - db
      - redis
    deploy:
      replicas: 3
      update_config:
        parallelism: 1
        order: start-first
      restart_policy:
        condition: on-failure
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3

  db:
    image: postgres:15
    environment:
      - POSTGRES_USER=${DB_USER}
      - POSTGRES_PASSWORD=${DB_PASSWORD}
      - POSTGRES_DB=${DB_NAME}
    volumes:
      - postgres-data:/var/lib/postgresql/data
    deploy:
      restart_policy:
        condition: on-failure

  redis:
    image: redis:7
    volumes:
      - redis-data:/data
    deploy:
      restart_policy:
        condition: on-failure

volumes:
  postgres-data:
  redis-data:
```

## Security Scanning

### Container Scanning
```yaml
name: Container Security

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]

jobs:
  scan:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Build image
        run: docker build -t app:test .
      
      - name: Run Trivy vulnerability scanner
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: 'app:test'
          format: 'sarif'
          output: 'trivy-results.sarif'
          severity: 'CRITICAL,HIGH'
      
      - name: Upload Trivy scan results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: 'trivy-results.sarif'
```

## Build Optimization

### Layer Caching
- Dependencies layer
- Source code layer
- Configuration layer

### Multi-Stage Benefits
- Smaller final image
- No build tools in production
- Reduced attack surface

### BuildKit Features Used
- Parallel building
- Build caching
- Cross-platform compilation
- Layer optimization

## Registry Integration

### GitHub Container Registry
```bash
# Login to GHCR
echo $CR_PAT | docker login ghcr.io -u USERNAME --password-stdin

# Tag image
docker tag app:latest ghcr.io/username/app:latest

# Push image
docker push ghcr.io/username/app:latest
```

### Security Signing
```bash
# Install cosign
brew install cosign

# Generate keypair
cosign generate-key-pair

# Sign image
cosign sign --key cosign.key ghcr.io/username/app:latest

# Verify image
cosign verify --key cosign.pub ghcr.io/username/app:latest
```

## Best Practices

### Security
- Use distroless base images
- Run as non-root user
- Implement health checks
- Regular security scanning
- Image signing

### Performance
- Layer optimization
- Build caching
- Multi-stage builds
- Proper base image selection

### Maintainability
- Clear documentation
- Version tagging
- CI/CD integration
- Automated testing 