# vx Docker image
# Multi-stage build for minimal image size

# ============================================
# Stage 1: Build stage (optional, for source builds)
# ============================================
FROM rust:1.93.1-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

WORKDIR /build
COPY . .

RUN cargo build --release --locked --bin vx && \
    strip target/release/vx

# ============================================
# Stage 2: Runtime stage
# ============================================
FROM alpine:3.23 AS runtime

LABEL org.opencontainers.image.title="vx"
LABEL org.opencontainers.image.description="Universal version manager for developer tools"
LABEL org.opencontainers.image.url="https://github.com/loonghao/vx"
LABEL org.opencontainers.image.source="https://github.com/loonghao/vx"
LABEL org.opencontainers.image.vendor="loonghao"
LABEL org.opencontainers.image.licenses="MIT"

# Install runtime dependencies
RUN apk add --no-cache \
    ca-certificates \
    git \
    curl \
    bash \
    && rm -rf /var/cache/apk/*

# Create non-root user
RUN addgroup -g 1000 vx && \
    adduser -D -s /bin/bash -u 1000 -G vx vx

# Copy binary from builder or external source
# When building from source: COPY --from=builder /build/target/release/vx /usr/local/bin/vx
# When using pre-built binary: COPY vx /usr/local/bin/vx
ARG VX_BINARY_SOURCE=builder
COPY --from=builder /build/target/release/vx /usr/local/bin/vx

# Ensure binary is executable
RUN chmod +x /usr/local/bin/vx

# Create vx directories
RUN mkdir -p /home/vx/.vx && \
    chown -R vx:vx /home/vx

# Switch to non-root user
USER vx
WORKDIR /home/vx

# Verify installation
RUN vx --version

ENTRYPOINT ["/usr/local/bin/vx"]
CMD ["--help"]
