FROM rust:1.75-slim as builder

WORKDIR /usr/src/cli-lynx-fm
COPY . .

# Build the application with release profile
RUN cargo build --release

# Create a smaller runtime image
FROM debian:bookworm-slim

# Install dependencies for audio playback
RUN apt-get update && apt-get install -y \
    libasound2 \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy the binary from the builder stage
COPY --from=builder /usr/src/cli-lynx-fm/target/release/lynx-fm /usr/local/bin/lynx-fm

# Create a directory for configuration
RUN mkdir -p /root/.lynx-fm

# Set the entrypoint
ENTRYPOINT ["lynx-fm"]

# Default command (can be overridden)
CMD ["--help"] 