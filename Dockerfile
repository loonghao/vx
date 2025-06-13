# Multi-stage build for vx
FROM alpine:latest

# Install runtime dependencies in a single layer for faster builds
RUN apk add --no-cache --update \
    ca-certificates \
    git \
    curl \
    bash \
    && rm -rf /var/cache/apk/*

# Create a non-root user
RUN addgroup -g 1000 vx && \
    adduser -D -s /bin/bash -u 1000 -G vx vx

# Copy the binary from GoReleaser
COPY vx /usr/local/bin/vx

# Make sure the binary is executable
RUN chmod +x /usr/local/bin/vx

# Switch to non-root user
USER vx

# Set working directory
WORKDIR /home/vx

# Set entrypoint
ENTRYPOINT ["/usr/local/bin/vx"]
CMD ["--help"]
